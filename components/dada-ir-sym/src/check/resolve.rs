//! Code to resolve inference variables to concrete types and permissions.

use std::collections::hash_map::Entry;

use dada_ir_ast::diagnostic::{Diagnostic, Err, Reported};
use dada_util::Map;

use crate::ir::{
    classes::SymAggregateStyle,
    indices::InferVarIndex,
    subst::Subst,
    types::{SymGenericTerm, SymPerm, SymTy, SymTyKind},
};

use super::{
    Env,
    inference::{Direction, InferVarKind},
    red::RedTy,
};

pub struct Resolver<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env mut Env<'db>,

    // For a type inference variable, the algorithm is
    // that we prefer the lower bound if present,
    // then the upper bound if not, and fallback to
    // `!` (never).
    //
    // Types can include other types which must be
    // recursively resolved. To account for the possibility
    // of cycles, we insert `Err` when we begin resolving
    // a type's value, and `Ok` when we finish.
    // If we encounter a cycle, we error.
    memoized_ty: Map<InferVarIndex, Result<SymTy<'db>, ResolverCycle>>,

    // For a permission inference variable, the algorithm is
    // that we prefer the lower bound if present,
    // then the upper bound if not, and fallback to
    // `my`.
    //
    // Permissions cannot encounter cycles since `RedPerm`
    // bounds do not reference other inference variables or
    // have recursive structure.
    memoized_perm: Map<InferVarIndex, SymPerm<'db>>,
}

impl<'env, 'db> Resolver<'env, 'db> {
    pub fn new(env: &'env mut Env<'db>) -> Self {
        assert!(
            env.runtime().check_complete(),
            "resolution is only possible once type constraints are known"
        );

        Self {
            db: env.db(),
            env,
            memoized_ty: Default::default(),
            memoized_perm: Default::default(),
        }
    }

    /// Return a version of `term` in which all inference variables are (deeply) removed.
    pub fn resolve<T>(&mut self, term: T) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>>,
    {
        let mut bound_vars = self.env.bound_vars();
        term.resolve_infer_var(self.db, &mut bound_vars, |infer| {
            match self.resolve_infer_var(infer) {
                Ok(v) => Some(v),
                Result::Err(error) => Some(SymGenericTerm::err(self.db, self.report(infer, error))),
            }
        })
    }

    /// Resolve an inference variable to a generic term, given the variance of the location in which it appears
    fn resolve_infer_var(
        &mut self,
        infer: InferVarIndex,
    ) -> Result<SymGenericTerm<'db>, ResolverCycle> {
        match self.env.infer_var_kind(infer) {
            InferVarKind::Type => Ok(self.resolve_ty_var(infer)?.into()),
            InferVarKind::Perm => Ok(self.resolve_perm_var(infer).into()),
        }
    }

    fn resolve_ty_var(&mut self, infer: InferVarIndex) -> Result<SymTy<'db>, ResolverCycle> {
        match self.memoized_ty.entry(infer) {
            Entry::Occupied(entry) => {
                return *entry.get();
            }
            Entry::Vacant(entry) => {
                entry.insert(Err(ResolverCycle));
            }
        }

        let ty = if let Some(t) = self.bounding_ty(infer, Direction::FromBelow)? {
            t
        } else if let Some(t) = self.bounding_ty(infer, Direction::FromAbove)? {
            t
        } else {
            // we always insert a fallback bound, so this should not occur
            panic!("found no inference bounds, odd")
        };

        self.memoized_ty.insert(infer, Ok(ty));
        Ok(ty)
    }

    /// Return the bounding type on the type inference variable `v` from the given `direction`.
    fn bounding_ty(
        &mut self,
        infer: InferVarIndex,
        direction: Direction,
    ) -> Result<Option<SymTy<'db>>, ResolverCycle> {
        let db = self.env.db();

        let bound = self.env.red_bound(infer, direction).peek_ty();

        let Some((red_ty, _)) = bound else {
            return Ok(None);
        };

        let apply_perm = |this: &mut Self, sym_ty: SymTy<'db>| {
            let perm_infer = this.env.perm_infer(infer);
            let sym_perm = this.resolve_perm_var(perm_infer);
            sym_perm.apply_to(db, sym_ty)
        };

        Ok(Some(match red_ty {
            RedTy::Error(reported) => SymTy::err(db, reported),
            RedTy::Named(name, args) => {
                let args = self.resolve(args);
                let ty = SymTy::new(db, SymTyKind::Named(name, args));
                match name.style(db) {
                    SymAggregateStyle::Struct => ty,
                    SymAggregateStyle::Class => apply_perm(self, ty),
                }
            }
            RedTy::Never => SymTy::new(db, SymTyKind::Never),
            RedTy::Infer(_) => panic!("infer bound cannot be another infer"),
            RedTy::Var(var) => apply_perm(self, SymTy::new(db, SymTyKind::Var(var))),
            RedTy::Perm => panic!("infer bound cannot be a perm"),
        }))
    }

    fn resolve_perm_var(&mut self, infer: InferVarIndex) -> SymPerm<'db> {
        if let Some(perm) = self.memoized_perm.get(&infer) {
            return *perm;
        }

        let perm = if let Some(t) = self.bounding_perm(infer, Direction::FromBelow) {
            t
        } else if let Some(t) = self.bounding_perm(infer, Direction::FromAbove) {
            t
        } else {
            // we always insert a fallback bound, so this should not occur
            panic!("found no inference bounds, odd")
        };

        self.memoized_perm.insert(infer, perm);
        perm
    }

    /// Return the bounding perm on the permission inference variable `v` from the given `direction`.
    fn bounding_perm(&self, infer: InferVarIndex, direction: Direction) -> Option<SymPerm<'db>> {
        let runtime = self.env.runtime().clone();
        runtime.with_inference_var_data(infer, |data| match data.red_perm_bound(direction) {
            Some((bound, _)) => Some(bound.to_sym_perm(self.db)),
            None => None,
        })
    }

    fn report(&self, infer: InferVarIndex, _err: ResolverCycle) -> Reported {
        let span = self.env.infer_var_span(infer);
        Diagnostic::error(self.db, span, "cyclic bounds found for inference variable")
            .report(self.db)
    }
}

#[derive(Copy, Clone)]
struct ResolverCycle;

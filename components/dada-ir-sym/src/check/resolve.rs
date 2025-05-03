//! Code to resolve inference variables to concrete types and permissions.

use dada_ir_ast::diagnostic::{Diagnostic, Err, Level, Reported};
use dada_util::Set;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    subst::Subst,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind},
};

use super::{
    Env,
    inference::{Direction, InferVarKind},
    predicates::Predicate,
    red::{RedChain, RedLink, RedPerm, RedTy},
};

pub struct Resolver<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env mut Env<'db>,
    var_stack: Set<InferVarIndex>,
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
            var_stack: Default::default(),
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
    ) -> Result<SymGenericTerm<'db>, ResolverError<'db>> {
        if self.var_stack.insert(infer) {
            let mut compute_result = || -> Result<SymGenericTerm<'db>, ResolverError<'db>> {
                match self.env.infer_var_kind(infer) {
                    InferVarKind::Type => {
                        if let Some(ty) = self.bounding_ty(infer, Direction::FromBelow)? {
                            Ok(ty.into())
                        } else if let Some(ty) = self.bounding_ty(infer, Direction::FromAbove)? {
                            Ok(ty.into())
                        } else {
                            Err(ResolverError::NoBounds)
                        }
                    }

                    InferVarKind::Perm => {
                        Ok(self.bounding_perm(infer, Direction::FromBelow)?.into())
                    }
                }
            };

            let result = compute_result();
            self.var_stack.remove(&infer);
            result
        } else {
            Err(ResolverError::Cycle)
        }
    }

    fn report(&self, infer: InferVarIndex, err: ResolverError<'db>) -> Reported {
        let span = self.env.infer_var_span(infer);
        match err {
            ResolverError::NoBounds => {
                Diagnostic::error(self.db, span, "no bounds found for inference variable")
                    .report(self.db)
            }
            ResolverError::Cycle => {
                Diagnostic::error(self.db, span, "cyclic bounds found for inference variable")
                    .report(self.db)
            }
            ResolverError::Irreconciliable { left, right } => {
                self.report_irreconciliable_error(infer, left, right)
            }
        }
    }

    fn report_irreconciliable_error<T: Err<'db>>(
        &self,
        infer: InferVarIndex,
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
    ) -> T {
        // FIXME: This error stinks. We need better spans threaded through inference to do better, though.
        // This would be an interesting place to deply AI.

        let (infer_var_kind, infer_var_span) = self
            .env
            .runtime()
            .with_inference_var_data(infer, |data| (data.kind(), data.span()));

        let message = match infer_var_kind {
            InferVarKind::Type => "cannot infer a type due to conflicting constraints",
            InferVarKind::Perm => "cannot infer a permission due to conflicting constraints",
        };
        T::err(
            self.db,
            Diagnostic::error(self.db, infer_var_span, message)
                .label(
                    self.db,
                    Level::Error,
                    infer_var_span,
                    format!("constraint 1 is {left:?}"),
                )
                .label(
                    self.db,
                    Level::Error,
                    infer_var_span,
                    format!("constraint 2 is {right:?}"),
                )
                .report(self.db),
        )
    }

    /// Return the bounding type on the type inference variable `v` from the given `direction`.
    fn bounding_ty(
        &mut self,
        infer: InferVarIndex,
        direction: Direction,
    ) -> Result<Option<SymTy<'db>>, ResolverError<'db>> {
        let db = self.env.db();

        let bound = self.env.red_ty_bound(infer, direction).peek();

        let Some((red_ty, _)) = bound else {
            return Ok(None);
        };

        let sym_ty = match red_ty {
            RedTy::Error(reported) => SymTy::err(db, reported),
            RedTy::Named(name, args) => {
                let args = self.resolve(args);
                SymTy::new(db, SymTyKind::Named(name, args))
            }
            RedTy::Never => SymTy::new(db, SymTyKind::Never),
            RedTy::Infer(_) => panic!("infer bound cannot be another infer"),
            RedTy::Var(var) => SymTy::new(db, SymTyKind::Var(var)),
            RedTy::Perm => panic!("infer bound cannot be a perm"),
        };

        let perm_infer = self.env.perm_infer(infer);
        let sym_perm = self.bounding_perm(perm_infer, direction)?;
        Ok(Some(SymTy::new(db, SymTyKind::Perm(sym_perm, sym_ty))))
    }

    /// Return the bounding perm on the permission inference variable `v` from the given `direction`.
    fn bounding_perm(
        &mut self,
        infer: InferVarIndex,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
        let runtime = self.env.runtime().clone();
        runtime.with_inference_var_data(infer, |data| {
            let chains = data.red_perm_bounds(direction);
            // XXX what do we do here -- we have multiple bounds that must hold which is NOT the same
            // as an `our` bound`
            self.merge_lien_chains(chains.iter().map(|pair| &pair.0), direction)
        })
    }

    fn red_perm_to_perm(&self, perm: RedPerm<'db>) -> SymPerm<'db> {
        perm.chains(self.db)
            .iter()
            .map(|&chain| self.red_chain_to_perm(chain))
            .reduce(|perm1, perm2| SymPerm::or(self.db, perm1, perm2))
            .unwrap()
    }

    fn red_chain_to_perm(&self, chain: RedChain<'db>) -> SymPerm<'db> {
        chain
            .links(self.db)
            .iter()
            .map(|&link| self.red_link_to_perm(link))
            .reduce(|perm1, perm2| SymPerm::apply(self.db, perm1, perm2))
            .unwrap_or_else(|| SymPerm::my(self.db))
    }

    fn red_link_to_perm(&self, link: RedLink<'db>) -> SymPerm<'db> {
        match link {
            RedLink::Our => SymPerm::our(self.db),
            RedLink::Ref(_, place) => SymPerm::shared(self.db, vec![place]),
            RedLink::Mut(_, place) => SymPerm::leased(self.db, vec![place]),
            RedLink::Var(v) => SymPerm::var(self.db, v),
        }
    }
}

enum ResolverError<'db> {
    NoBounds,

    Cycle,

    Irreconciliable {
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
    },
}

impl<'db> ResolverError<'db> {
    pub fn irreconciliable(
        left: impl Into<SymGenericTerm<'db>>,
        right: impl Into<SymGenericTerm<'db>>,
    ) -> Self {
        ResolverError::Irreconciliable {
            left: left.into(),
            right: right.into(),
        }
    }
}

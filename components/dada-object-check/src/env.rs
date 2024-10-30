use std::sync::Arc;

use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_ir_sym::{
    binder::BoundTerm,
    scope::Scope,
    subst::SubstWith,
    symbol::{SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymPerm, SymTy},
};
use dada_util::Map;
use futures::{Stream, StreamExt};

use crate::{
    bound::{Bound, InferenceVarBounds},
    check::Check,
    object_ir::{IntoObjectIr, ObjectGenericTerm, ObjectTy, ObjectTyKind},
    subobject::{require_assignable_object_type, require_numeric_type, require_sub_object_type},
    universe::Universe,
};

#[derive(Clone)]
pub struct Env<'db> {
    universe: Universe,

    /// Lexical scope for name resolution
    pub scope: Arc<Scope<'db, 'db>>,

    /// Universe of each free variable that is in scope.
    variable_universes: Arc<Map<SymVariable<'db>, Universe>>,

    /// For place variables, keep their type.
    variable_tys: Arc<Map<SymVariable<'db>, SymTy<'db>>>,

    /// If `None`, not type checking a function or method.
    pub return_ty: Option<SymTy<'db>>,
}

impl<'db> Env<'db> {
    /// Create an empty environment
    pub fn new(scope: Scope<'db, 'db>) -> Self {
        Self {
            universe: Universe::ROOT,
            scope: Arc::new(scope),
            variable_tys: Default::default(),
            variable_universes: Default::default(),
            return_ty: Default::default(),
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// Open the given symbols as universally quantified.
    /// Creates a new universe.
    pub fn open_universally<T>(
        &mut self,
        check: &Check<'db>,
        variables: &[SymVariable<'db>],
        value: &T,
    ) -> T::LeafTerm
    where
        T: BoundTerm<'db>,
    {
        let db = check.db;

        match value.as_binder() {
            Err(leaf) => {
                return leaf.identity();
            }

            Ok(binder) => {
                self.increment_universe();
                Arc::make_mut(&mut self.variable_universes)
                    .extend(binder.variables.iter().map(|&v| (v, self.universe)));

                self.open_universally(check, variables, &binder.bound_value)
            }
        }
    }

    /// Create a substitution for `binder` consisting of inference variables
    pub fn existential_substitution(
        &self,
        check: &Check<'db>,
        span: Span<'db>,
        variables: &[SymVariable<'db>],
    ) -> Vec<SymGenericTerm<'db>> {
        let db = check.db;
        variables
            .iter()
            .map(|&var| self.fresh_inference_var(check, var.kind(db), span))
            .collect()
    }

    // Modify this environment to put it in a new universe.
    pub fn increment_universe(&mut self) {
        self.universe = self.universe.next();
    }

    /// Inserts a new program variable into the environment for later lookup.
    pub fn insert_program_variable(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        Arc::make_mut(&mut self.scope).push_link(lv);
        Arc::make_mut(&mut self.variable_tys).insert(lv, ty);
    }

    /// Set the return type of the current function.
    pub fn set_return_ty(&mut self, ty: SymTy<'db>) {
        self.return_ty = Some(ty);
    }

    /// Returns the type of the given variable.
    ///
    /// # Panics
    ///
    /// If the variable is not present.
    pub fn variable_ty(&self, lv: SymVariable<'db>) -> SymTy<'db> {
        self.variable_tys.get(&lv).copied().unwrap()
    }

    pub fn fresh_inference_var(
        &self,
        check: &Check<'db>,
        kind: SymGenericKind,
        span: Span<'db>,
    ) -> SymGenericTerm<'db> {
        check.fresh_inference_var(kind, self.universe, span)
    }

    pub fn fresh_ty_inference_var(&self, check: &Check<'db>, span: Span<'db>) -> SymTy<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type, span)
            .assert_type(check.db)
    }

    pub fn fresh_object_ty_inference_var(
        &self,
        check: &Check<'db>,
        span: Span<'db>,
    ) -> ObjectTy<'db> {
        self.fresh_ty_inference_var(check, span)
            .into_object_ir(check.db)
    }

    pub fn fresh_perm_inference_var(&self, check: &Check<'db>, span: Span<'db>) -> SymPerm<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type, span)
            .assert_perm(check.db)
    }

    pub fn require_assignable_object_type(
        &self,
        check: &Check<'db>,
        value_span: Span<'db>,
        value_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
        place_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = check.db;
        let value_ty = value_ty.into_object_ir(db);
        let place_ty = value_ty.into_object_ir(db);
        check.defer(self, value_span, move |check, env| async move {
            match require_assignable_object_type(&check, &env, value_span, value_ty, place_ty).await
            {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_sub_object_type(
        &self,
        check: &Check<'db>,
        span: Span<'db>,
        sub_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
        sup_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = check.db;
        let value_ty = sub_ty.into_object_ir(db);
        let place_ty = value_ty.into_object_ir(db);
        check.defer(self, span, move |check, env| async move {
            match require_sub_object_type(&check, &env, span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_numeric_type(
        &self,
        check: &Check<'db>,
        span: Span<'db>,
        ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = check.db;
        let ty = ty.into_object_ir(db);
        check.defer(self, span, move |check, env| async move {
            match require_numeric_type(&check, &env, span, ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    /// Check whether any type in `tys` is known to be never (or error).
    /// If so, do nothing.
    /// Otherwise, if no type in `tys` is known to be never, invoke `op` (asynchronously).
    pub fn if_not_never(
        &self,
        check: &Check<'db>,
        span: Span<'db>,
        tys: &[ObjectTy<'db>],
        op: impl async FnOnce(Check<'db>, Env<'db>) + 'db,
    ) {
        let tys = tys.to_vec();
        check.defer(self, span, move |check, env: Env<'db>| async move {
            'next_ty: for ty in tys {
                'next_bound: while let Some(bound) = env.bounds(&check, ty).next().await {
                    let bound_ty = bound.into_term();
                    match bound_ty.kind(check.db) {
                        ObjectTyKind::Never => return,
                        ObjectTyKind::Error(reported) => return,
                        ObjectTyKind::Infer(sym_infer_var_index) => continue 'next_bound,
                        ObjectTyKind::Named(..) | ObjectTyKind::Var(_) => continue 'next_ty,
                    }
                }
            }

            op(check, env).await
        })
    }

    pub fn bounds(
        &self,
        check: &Check<'db>,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = Bound<ObjectTy<'db>>> + 'db {
        let db = check.db;
        if let &ObjectTyKind::Infer(inference_var) = ty.kind(db) {
            <InferenceVarBounds<'db, ObjectGenericTerm<'db>>>::new(check, inference_var)
                .map(|b| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(Bound::LowerBound(ty))).boxed_local()
        }
    }

    pub fn describe_ty<'a, 'chk>(
        &'a self,
        check: &'a Check<'db>,
        ty: ObjectTy<'db>,
    ) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }
}

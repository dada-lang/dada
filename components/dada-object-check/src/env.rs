use std::sync::Arc;

use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_ir_sym::{
    binder::BoundTerm,
    scope::Scope,
    subst::SubstWith,
    symbol::{SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymTy},
};
use dada_util::Map;
use futures::{Stream, StreamExt};

use crate::{
    bound::{Direction, TransitiveBounds},
    check::Runtime,
    object_ir::{ObjectExpr, ObjectGenericTerm, ObjectTy, ObjectTyKind, ToObjectIr},
    subobject::{
        require_assignable_object_type, require_numeric_type, require_sub_object_type, Expected,
    },
    universe::Universe,
};

#[derive(Clone)]
pub(crate) struct Env<'db> {
    universe: Universe,

    /// Reference to the runtime
    runtime: Runtime<'db>,

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
    pub(crate) fn new(runtime: &Runtime<'db>, scope: Scope<'db, 'db>) -> Self {
        Self {
            universe: Universe::ROOT,
            runtime: runtime.clone(),
            scope: Arc::new(scope),
            variable_tys: Default::default(),
            variable_universes: Default::default(),
            return_ty: Default::default(),
        }
    }

    /// Extract the scope from the environment.
    ///
    /// # Panics
    ///
    /// If the scope has an outstanding reference.
    #[track_caller]
    pub fn into_scope(self) -> Scope<'db, 'db> {
        Arc::into_inner(self.scope).unwrap()
    }

    #[expect(dead_code)]
    pub(crate) fn universe(&self) -> Universe {
        self.universe
    }

    /// Get the database
    pub fn db(&self) -> &'db dyn crate::Db {
        self.runtime.db
    }

    /// Access the lower-level type checking runtime
    pub fn runtime(&self) -> &Runtime<'db> {
        &self.runtime
    }

    /// Open the given symbols as universally quantified.
    /// Creates a new universe.
    pub fn open_universally<T>(&mut self, runtime: &Runtime<'db>, value: &T) -> T::LeafTerm
    where
        T: BoundTerm<'db>,
    {
        match value.as_binder() {
            Err(leaf) => {
                return leaf.identity();
            }

            Ok(binder) => {
                self.increment_universe();
                Arc::make_mut(&mut self.variable_universes)
                    .extend(binder.variables.iter().map(|&v| (v, self.universe)));

                self.open_universally(runtime, &binder.bound_value)
            }
        }
    }

    /// Create a substitution for `binder` consisting of inference variables
    pub fn existential_substitution(
        &self,
        span: Span<'db>,
        variables: &[SymVariable<'db>],
    ) -> Vec<SymGenericTerm<'db>> {
        let db = self.db();
        variables
            .iter()
            .map(|&var| self.fresh_inference_var(var.kind(db), span))
            .collect()
    }

    // Modify this environment to put it in a new universe.
    pub fn increment_universe(&mut self) {
        self.universe = self.universe.next();
    }

    /// Sets the type for a program variable that is in scope already.
    pub fn set_program_variable_ty(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        assert!(
            self.scope.generic_sym_in_scope(self.db(), lv),
            "variable `{lv:?}` not in scope"
        );
        assert!(
            !self.variable_tys.contains_key(&lv),
            "variable `{lv:?}` already has a type"
        );
        Arc::make_mut(&mut self.variable_tys).insert(lv, ty);
    }

    /// Extends the scope with a new program variable.
    /// You still need to call [`Self::set_program_variable_ty`][] after calling this function to set the type.
    pub fn push_program_variable_with_ty(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        Arc::make_mut(&mut self.scope).push_link(lv);
        self.set_program_variable_ty(lv, ty);
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
        kind: SymGenericKind,
        span: Span<'db>,
    ) -> SymGenericTerm<'db> {
        self.runtime.fresh_inference_var(kind, self.universe, span)
    }

    pub fn fresh_ty_inference_var(&self, span: Span<'db>) -> SymTy<'db> {
        self.fresh_inference_var(SymGenericKind::Type, span)
            .assert_type(self.db())
    }

    pub fn fresh_object_ty_inference_var(&self, span: Span<'db>) -> ObjectTy<'db> {
        self.fresh_ty_inference_var(span).to_object_ir(self)
    }

    pub fn require_assignable_object_type(
        &self,
        value_span: Span<'db>,
        value_ty: impl ToObjectIr<'db, Object = ObjectTy<'db>>,
        place_ty: impl ToObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let value_ty = value_ty.to_object_ir(self);
        let place_ty = place_ty.to_object_ir(self);
        self.runtime.defer(self, value_span, move |env| async move {
            match require_assignable_object_type(&env, value_span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_equal_object_types(
        &self,
        span: Span<'db>,
        expected_ty: impl ToObjectIr<'db, Object = ObjectTy<'db>>,
        found_ty: impl ToObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let expected_ty = expected_ty.to_object_ir(self);
        let found_ty = found_ty.to_object_ir(self);
        self.runtime.defer(self, span, move |env| async move {
            match require_sub_object_type(&env, Expected::Lower, span, expected_ty, found_ty).await
            {
                Ok(()) => (),
                Err(Reported(_)) => return,
            }

            match require_sub_object_type(&env, Expected::Upper, span, found_ty, expected_ty).await
            {
                Ok(()) => (),
                Err(Reported(_)) => return,
            }
        })
    }

    pub fn require_numeric_type(
        &self,
        span: Span<'db>,
        ty: impl ToObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let ty = ty.to_object_ir(self);
        self.runtime.defer(self, span, move |env| async move {
            match require_numeric_type(&env, span, ty).await {
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

        span: Span<'db>,
        tys: &[ObjectTy<'db>],
        op: impl async FnOnce(Env<'db>) + 'db,
    ) {
        let tys = tys.to_vec();
        self.runtime
            .defer(self, span, move |env: Env<'db>| async move {
                'next_ty: for ty in tys {
                    let mut bounds = env.transitive_lower_bounds(ty);
                    'next_bound: while let Some(bound_ty) = bounds.next().await {
                        match bound_ty.kind(env.db()) {
                            ObjectTyKind::Never => return,
                            ObjectTyKind::Error(_) => return,
                            ObjectTyKind::Infer(_) => continue 'next_bound,
                            ObjectTyKind::Named(..) | ObjectTyKind::Var(_) => continue 'next_ty,
                        }
                    }
                }

                op(env).await
            })
    }

    pub fn transitive_lower_bounds(
        &self,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = ObjectTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::LowerBoundedBy)
    }

    pub fn transitive_upper_bounds(
        &self,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = ObjectTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::UpperBoundedBy)
    }

    pub fn transitive_bounds(
        &self,
        ty: ObjectTy<'db>,
        direction: Direction,
    ) -> impl Stream<Item = ObjectTy<'db>> + 'db {
        let db = self.db();
        if let &ObjectTyKind::Infer(inference_var) = ty.kind(db) {
            TransitiveBounds::new(&self.runtime, direction, inference_var)
                .map(|b: ObjectGenericTerm<'db>| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(ty)).boxed_local()
        }
    }

    pub fn describe_ty<'a, 'chk>(&'a self, ty: ObjectTy<'db>) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }

    pub(crate) fn defer(&self, span: Span<'db>, op: impl async FnOnce(Self) + 'db) {
        self.runtime.defer(self, span, op)
    }

    pub(crate) fn require_expr_has_bool_ty(&self, expr: ObjectExpr<'db>) {
        let db = self.db();
        let boolean_ty = ObjectTy::boolean(db);
        self.require_assignable_object_type(expr.span(db), expr.ty(db), boolean_ty);
    }
}

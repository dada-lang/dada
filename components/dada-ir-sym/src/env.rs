use std::sync::Arc;

use crate::{
    indices::{FromInfer, InferVarIndex},
    ir::binder::BoundTerm,
    scope::Scope,
    subst::SubstWith,
    symbol::{SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyKind},
    CheckInEnv,
};
use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_util::Map;
use futures::{Stream, StreamExt};

use crate::{
    bound::{Direction, TransitiveBounds},
    check::Runtime,
    ir::exprs::SymExpr,
    subobject::{require_assignable_type, require_numeric_type, require_subtype, Expected},
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

    /// Object type for each in-scope variable.
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

    /// Convenience function for invoking `to_object_ir`.
    /// We have to do a bit of a "dance" because `to_object_ir` needs a mutable reference to a shared reference.
    pub fn symbolize<I>(&self, i: I) -> I::Output
    where
        I: CheckInEnv<'db>,
    {
        let mut env = self;
        i.check_in_env(&mut env)
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
            .map(|&var| self.fresh_inference_var_term(var.kind(db), span))
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

    pub fn return_ty(&self) -> Option<SymTy<'db>> {
        self.return_ty
    }

    /// Returns the type of the given variable.
    ///
    /// # Panics
    ///
    /// If the variable is not present.
    pub fn variable_ty(&self, lv: SymVariable<'db>) -> SymTy<'db> {
        self.variable_tys.get(&lv).copied().unwrap()
    }

    pub fn fresh_inference_var(&self, kind: SymGenericKind, span: Span<'db>) -> InferVarIndex {
        self.runtime.fresh_inference_var(kind, self.universe, span)
    }

    /// A fresh term with an inference variable of the given kind.
    pub fn fresh_inference_var_term(
        &self,
        kind: SymGenericKind,
        span: Span<'db>,
    ) -> SymGenericTerm<'db> {
        match kind {
            SymGenericKind::Type => SymGenericTerm::Type(SymTy::infer(
                self.db(),
                self.fresh_inference_var(kind, span),
            )),
            SymGenericKind::Perm => SymGenericTerm::Perm(SymPerm::infer(
                self.db(),
                self.fresh_inference_var(kind, span),
            )),
            SymGenericKind::Place => SymGenericTerm::Place(SymPlace::infer(
                self.db(),
                self.fresh_inference_var(kind, span),
            )),
        }
    }

    pub fn fresh_ty_inference_var(&self, span: Span<'db>) -> SymTy<'db> {
        SymTy::infer(
            self.db(),
            self.fresh_inference_var(SymGenericKind::Type, span),
        )
    }

    pub fn require_assignable_object_type(
        &self,
        value_span: Span<'db>,
        value_ty: impl CheckInEnv<'db, Output = SymTy<'db>>,
        place_ty: impl CheckInEnv<'db, Output = SymTy<'db>>,
    ) {
        let value_ty = self.symbolize(value_ty);
        let place_ty = self.symbolize(place_ty);
        self.runtime.defer(self, value_span, move |env| async move {
            match require_assignable_type(&env, value_span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_equal_object_types(
        &self,
        span: Span<'db>,
        expected_ty: impl CheckInEnv<'db, Output = SymTy<'db>>,
        found_ty: impl CheckInEnv<'db, Output = SymTy<'db>>,
    ) {
        let expected_ty = self.symbolize(expected_ty);
        let found_ty = self.symbolize(found_ty);
        self.runtime.defer(self, span, move |env| async move {
            match require_subtype(&env, Expected::Lower, span, expected_ty, found_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => return,
            }

            match require_subtype(&env, Expected::Upper, span, found_ty, expected_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => return,
            }
        })
    }

    pub fn require_numeric_type(
        &self,
        span: Span<'db>,
        ty: impl CheckInEnv<'db, Output = SymTy<'db>>,
    ) {
        let ty = self.symbolize(ty);
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
        tys: &[SymTy<'db>],
        op: impl async FnOnce(Env<'db>) + 'db,
    ) {
        let _tys = tys.to_vec();
        self.runtime
            .defer(self, span, move |env: Env<'db>| async move {
                // FIXME: check for never
                op(env).await
            })
    }

    pub fn transitive_lower_bounds(&self, ty: SymTy<'db>) -> impl Stream<Item = SymTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::LowerBoundedBy)
    }

    pub fn transitive_upper_bounds(&self, ty: SymTy<'db>) -> impl Stream<Item = SymTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::UpperBoundedBy)
    }

    pub fn transitive_bounds(
        &self,
        ty: SymTy<'db>,
        direction: Direction,
    ) -> impl Stream<Item = SymTy<'db>> + 'db {
        let db = self.db();
        if let &SymTyKind::Infer(inference_var) = ty.kind(db) {
            TransitiveBounds::new(&self.runtime, direction, inference_var)
                .map(|b: SymGenericTerm<'db>| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(ty)).boxed_local()
        }
    }

    pub fn describe_ty<'a, 'chk>(&'a self, ty: SymTy<'db>) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }

    pub(crate) fn defer(&self, span: Span<'db>, op: impl async FnOnce(Self) + 'db) {
        self.runtime.defer(self, span, op)
    }

    pub(crate) fn require_expr_has_bool_ty(&self, expr: SymExpr<'db>) {
        let db = self.db();
        let boolean_ty = SymTy::boolean(db);
        self.require_assignable_object_type(expr.span(db), expr.ty(db), boolean_ty);
    }
}

pub(crate) trait EnvLike<'db> {
    fn db(&self) -> &'db dyn crate::Db;
    fn scope(&self) -> &Scope<'db, 'db>;
    fn variable_ty(&mut self, var: SymVariable<'db>) -> SymTy<'db>;
}

impl<'db> EnvLike<'db> for &Env<'db> {
    fn db(&self) -> &'db dyn crate::Db {
        Env::db(self)
    }

    fn scope(&self) -> &Scope<'db, 'db> {
        &self.scope
    }

    fn variable_ty(&mut self, var: SymVariable<'db>) -> SymTy<'db> {
        Env::variable_ty(self, var)
    }
}

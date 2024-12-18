use std::{cell::Cell, sync::Arc};

use crate::{
    check::scope::Scope,
    check::CheckInEnvLike,
    ir::binder::BoundTerm,
    ir::indices::{FromInfer, InferVarIndex},
    ir::subst::SubstWith,
    ir::types::{SymGenericKind, SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyKind},
    ir::variables::SymVariable,
};
use dada_ir_ast::{
    ast::AstTy,
    diagnostic::{Diagnostic, Err, Reported},
    span::Span,
};
use dada_util::{debug, Map};

use crate::{
    check::bound::{Direction, TransitiveBounds},
    check::runtime::Runtime,
    check::subobject::{require_assignable_type, require_numeric_type, require_subtype, Expected},
    check::universe::Universe,
    ir::exprs::SymExpr,
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

    /// Type for in-scope variables. Local variables
    /// are stored directly in symbolic form but function
    /// parameters are stored initially as AST types.
    /// Those types are symbolified lazily.
    /// See [`VariableType`] for details.
    variable_tys: Arc<Map<SymVariable<'db>, VariableTypeCell<'db>>>,

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

    /// Convenience function for invoking [`CheckInEnvLike::check_in_env_like`][].
    pub(super) async fn check<I>(&self, i: I) -> I::Output
    where
        I: CheckInEnvLike<'db>,
    {
        i.check_in_env_like(self).await
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
    #[allow(dead_code)]
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

    /// Sets the symbolic type for a program variable. Used during environment
    /// construction but typically you should use [`Self::push_program_variable_with_ty`]
    /// instead.
    pub fn set_variable_sym_ty(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        assert!(
            self.scope.generic_sym_in_scope(self.db(), lv),
            "variable `{lv:?}` not in scope"
        );
        assert!(
            !self.variable_tys.contains_key(&lv),
            "variable `{lv:?}` already has a type"
        );
        Arc::make_mut(&mut self.variable_tys).insert(lv, VariableTypeCell::symbolic(lv, ty));
    }

    /// Sets the AST type for a parameter that is in scope already.
    /// This AST type will be lazily symbolified when requested.
    pub fn set_variable_ast_ty(&mut self, lv: SymVariable<'db>, ty: AstTy<'db>) {
        assert!(
            self.scope.generic_sym_in_scope(self.db(), lv),
            "variable `{lv:?}` not in scope"
        );
        assert!(
            !self.variable_tys.contains_key(&lv),
            "variable `{lv:?}` already has a type"
        );
        Arc::make_mut(&mut self.variable_tys).insert(lv, VariableTypeCell::ast(lv, ty));
    }

    /// Extends the scope with a new program variable given its type.
    pub fn push_program_variable_with_ty(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        Arc::make_mut(&mut self.scope).push_link(lv);
        self.set_variable_sym_ty(lv, ty);
    }

    /// Set the return type of the current function.
    pub fn set_return_ty(&mut self, ty: SymTy<'db>) {
        self.return_ty = Some(ty);
    }

    #[expect(dead_code)]
    pub fn return_ty(&self) -> Option<SymTy<'db>> {
        self.return_ty
    }

    /// Returns the type of the given variable.
    ///
    /// # Panics
    ///
    /// If the variable is not present.
    pub async fn variable_ty(&self, lv: SymVariable<'db>) -> SymTy<'db> {
        self.variable_tys
            .get(&lv)
            .expect("variable not in scope")
            .get(self)
            .await
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

    pub(super) fn require_assignable_object_type(
        &self,
        value_span: Span<'db>,
        value_ty: SymTy<'db>,
        place_ty: SymTy<'db>,
    ) {
        debug!("defer require_assignable_object_type", value_ty, place_ty);
        self.runtime.defer(self, value_span, async move |env| {
            debug!("require_assignable_object_type", value_ty, place_ty);

            match require_assignable_type(&env, value_span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub(super) fn require_equal_object_types(
        &self,
        span: Span<'db>,
        expected_ty: SymTy<'db>,
        found_ty: SymTy<'db>,
    ) {
        debug!("defer require_equal_object_types", expected_ty, found_ty);
        self.runtime.defer(self, span, move |env| async move {
            debug!("require_equal_object_types", expected_ty, found_ty);

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

    pub(super) fn require_numeric_type(&self, span: Span<'db>, ty: SymTy<'db>) {
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

    pub fn transitive_lower_bounds(&self, ty: SymTy<'db>) -> TransitiveBounds<'db, SymTy<'db>> {
        self.transitive_bounds(ty, Direction::LowerBoundedBy)
    }

    pub fn transitive_upper_bounds(&self, ty: SymTy<'db>) -> TransitiveBounds<'db, SymTy<'db>> {
        self.transitive_bounds(ty, Direction::UpperBoundedBy)
    }

    pub fn transitive_bounds(
        &self,
        ty: SymTy<'db>,
        direction: Direction,
    ) -> TransitiveBounds<'db, SymTy<'db>> {
        let db = self.db();
        if let &SymTyKind::Infer(inference_var) = ty.kind(db) {
            TransitiveBounds::new(&self.runtime, direction, inference_var)
        } else {
            TransitiveBounds::just(&self.runtime, direction, ty)
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
}

impl<'db> EnvLike<'db> for Env<'db> {
    fn db(&self) -> &'db dyn crate::Db {
        Env::db(self)
    }

    fn scope(&self) -> &Scope<'db, 'db> {
        &self.scope
    }
}

#[derive(Clone)]
struct VariableTypeCell<'db> {
    lv: SymVariable<'db>,
    state: Cell<VariableType<'db>>,
}

impl<'db> VariableTypeCell<'db> {
    fn symbolic(lv: SymVariable<'db>, ty: SymTy<'db>) -> Self {
        Self {
            lv,
            state: Cell::new(VariableType::Symbolic(ty)),
        }
    }

    fn ast(lv: SymVariable<'db>, ty: AstTy<'db>) -> Self {
        Self {
            lv,
            state: Cell::new(VariableType::Ast(ty)),
        }
    }

    async fn get(&self, env: &Env<'db>) -> SymTy<'db> {
        match self.state.get() {
            VariableType::Ast(ast_ty) => {
                self.state.set(VariableType::InProgress(ast_ty));
                let sym_ty = ast_ty.check_in_env_like(env).await;
                // update state to symbolic unless it was already set to an error
                if let VariableType::InProgress(_) = self.state.get() {
                    self.state.set(VariableType::Symbolic(sym_ty));
                }
                sym_ty
            }
            VariableType::InProgress(ast_ty) => {
                let ty_err = SymTy::err(
                    env.db(),
                    Diagnostic::error(
                        env.db(),
                        ast_ty.span(env.db()),
                        format!("type of `{}` references itself", self.lv),
                    )
                    .report(env.db()),
                );
                self.state.set(VariableType::Symbolic(ty_err));
                ty_err
            }
            VariableType::Symbolic(sym_ty) => sym_ty,
        }
    }
}

/// The type of a variable.
#[derive(Copy, Clone)]
enum VariableType<'db> {
    /// AST form of the type is available and we have not yet begun to symbolify it.
    /// AST types are used when we introduce a set of variables, where each variable
    /// may refer to others as part of its type. In that case we don't know the right
    /// order to process the variables in so we have to do a depth-first search.
    ///
    /// e.g., in `fn foo(x: shared[y] String, y: my Vec[String])`, we could begin with
    /// `y` then `x`, but that is not clear at first. So instead we begin with `x`, mark it as
    /// in progress, and then to convert `y` to a symbolic expression, wind up converting
    /// the type of `y`. If `y` did refer to `x`, this would result in an error.
    Ast(AstTy<'db>),

    /// AST form of the type is available and we have begun symbolifying it.
    /// When in this state, a repeated request for the variable's type will report an error.
    InProgress(AstTy<'db>),

    /// Symbolic type is available. For local variables, we introduce the type directly
    /// in this form, but for parameters or other cases where there are a set of variables
    /// introduced at once, we have to begin with AST form.
    Symbolic(SymTy<'db>),
}

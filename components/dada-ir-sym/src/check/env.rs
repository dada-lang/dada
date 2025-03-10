use std::{cell::Cell, ops::AsyncFnOnce, panic::Location, sync::Arc};

use crate::{
    check::{
        debug::TaskDescription,
        scope::Scope,
        subtype::terms::{require_assignable_type, require_sub_terms},
    },
    ir::{
        binder::BoundTerm,
        indices::{FromInfer, InferVarIndex},
        subst::SubstWith,
        types::{
            Assumption, AssumptionKind, SymGenericKind, SymGenericTerm, SymPerm, SymTy, SymTyName,
            Variance,
        },
        variables::SymVariable,
    },
};
use dada_ir_ast::{
    ast::AstTy,
    diagnostic::{Diagnostic, Err, Reported},
    span::Span,
};
use dada_util::{Map, debug};

use crate::{check::runtime::Runtime, check::universe::Universe, ir::exprs::SymExpr};

use super::{
    CheckInEnv,
    debug::{LogHandle, ToEventArgument},
    inference::{InferVarKind, InferenceVarData},
    predicates::Predicate,
    report::{BooleanTypeRequired, OrElse},
    runtime::DeferResult,
    subtype::{is_future::require_future_type, is_numeric::require_numeric_type},
};

pub mod combinator;

pub(crate) struct Env<'db> {
    pub log: LogHandle<'db>,

    universe: Universe,

    /// Reference to the runtime
    runtime: Runtime<'db>,

    /// Lexical scope for name resolution
    pub scope: Arc<Scope<'db, 'db>>,

    /// Universe of each free generic variable that is in scope.
    variable_universes: Arc<Map<SymVariable<'db>, Universe>>,

    /// Type for in-scope variables. Local variables
    /// are stored directly in symbolic form but function
    /// parameters are stored initially as AST types.
    /// Those types are symbolified lazily.
    /// See [`VariableType`] for details.
    variable_tys: Arc<Map<SymVariable<'db>, VariableTypeCell<'db>>>,

    /// If `None`, not type checking a function or method.
    pub return_ty: Option<SymTy<'db>>,

    /// Assumptions declared
    assumptions: Arc<Vec<Assumption<'db>>>,
}

impl<'db> Env<'db> {
    /// Create an empty environment
    pub(crate) fn new(runtime: &Runtime<'db>, scope: Scope<'db, 'db>) -> Self {
        Self {
            log: runtime.root_log(),
            universe: Universe::ROOT,
            runtime: runtime.clone(),
            scope: Arc::new(scope),
            variable_tys: Default::default(),
            variable_universes: Default::default(),
            return_ty: Default::default(),
            assumptions: Arc::new(vec![]), // FIXME
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
    pub fn universe(&self) -> Universe {
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

    /// Create a new environment from this environment.
    /// The log will be adjusted per the `log` function.
    pub fn fork(&self, log: impl FnOnce(&LogHandle<'db>) -> LogHandle<'db>) -> Env<'db> {
        Env {
            log: log(&self.log),
            universe: self.universe,
            runtime: self.runtime.clone(),
            scope: self.scope.clone(),
            variable_universes: self.variable_universes.clone(),
            variable_tys: self.variable_tys.clone(),
            return_ty: self.return_ty,
            assumptions: self.assumptions.clone(),
        }
    }

    /// True if the given variable is declared to meet the given predicate.
    pub fn var_is_declared_to_be(&self, var: SymVariable<'db>, predicate: Predicate) -> bool {
        match predicate {
            Predicate::Copy => self.assumed(var, |kind| {
                matches!(
                    kind,
                    AssumptionKind::Copy | AssumptionKind::Our | AssumptionKind::Shared
                )
            }),
            Predicate::Move => self.assumed(var, |kind| {
                matches!(
                    kind,
                    AssumptionKind::Move | AssumptionKind::My | AssumptionKind::Leased
                )
            }),
            Predicate::Owned => self.assumed(var, |kind| {
                matches!(
                    kind,
                    AssumptionKind::Owned | AssumptionKind::My | AssumptionKind::Our
                )
            }),
            Predicate::Lent => self.assumed(var, |kind| {
                matches!(
                    kind,
                    AssumptionKind::Lent | AssumptionKind::Leased | AssumptionKind::Shared
                )
            }),
        }
    }

    fn assumed(&self, var: SymVariable<'db>, kind: impl Fn(AssumptionKind) -> bool) -> bool {
        self.assumptions
            .iter()
            .any(|a| a.var(self.db()) == var && kind(a.kind(self.db())))
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
        &mut self,
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
    pub async fn variable_ty(&mut self, lv: SymVariable<'db>) -> SymTy<'db> {
        let variable_tys = self.variable_tys.clone();
        variable_tys
            .get(&lv)
            .expect("variable not in scope")
            .get(self)
            .await
    }

    /// Create a fresh inference variable of the given kind.
    pub fn fresh_inference_var(&mut self, kind: SymGenericKind, span: Span<'db>) -> InferVarIndex {
        let data = match kind {
            SymGenericKind::Type => {
                let perm = self.fresh_inference_var(SymGenericKind::Perm, span);
                InferenceVarData::new_ty(self.universe, span, perm)
            }
            SymGenericKind::Perm => InferenceVarData::new_perm(self.universe, span),
            SymGenericKind::Place => panic!("inference variable of kind `Place` not supported"),
        };
        let infer = self.runtime.fresh_inference_var(data);
        infer
    }

    /// A fresh term with an inference variable of the given kind.
    pub fn fresh_inference_var_term(
        &mut self,
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
            SymGenericKind::Place => panic!("cannot create inference variable for place"),
        }
    }

    /// Create a fresh type inference variable.
    pub fn fresh_ty_inference_var(&mut self, span: Span<'db>) -> SymTy<'db> {
        SymTy::infer(
            self.db(),
            self.fresh_inference_var(SymGenericKind::Type, span),
        )
    }

    /// Spawn a subtask that will require `value_ty` be assignable to `place_ty`.
    #[track_caller]
    pub(super) fn spawn_require_assignable_type(
        &mut self,
        value_ty: SymTy<'db>,
        place_ty: SymTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) {
        debug!("defer require_assignable_object_type", value_ty, place_ty);
        let or_else = or_else.to_arc();
        self.runtime.spawn(
            self,
            TaskDescription::RequireAssignableType(value_ty, place_ty),
            async move |env| {
                debug!("require_assignable_object_type", value_ty, place_ty);

                match require_assignable_type(env, value_ty, place_ty, &or_else).await {
                    Ok(()) => (),
                    Err(Reported(_)) => (),
                }
            },
        )
    }

    /// Spawn a subtask that will require `expected_ty` be equal to `found_ty`.
    #[track_caller]
    pub(super) fn spawn_require_equal_types(
        &self,
        expected_ty: SymTy<'db>,
        found_ty: SymTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) {
        debug!("defer require_equal_object_types", expected_ty, found_ty);
        let or_else = or_else.to_arc();
        self.runtime.spawn(
            self,
            TaskDescription::RequireEqualTypes(expected_ty, found_ty),
            async move |env| {
                debug!("require_equal_object_types", expected_ty, found_ty);

                env.require_both(
                    async |env| {
                        require_sub_terms(env, expected_ty.into(), found_ty.into(), &or_else).await
                    },
                    async |env| {
                        require_sub_terms(env, found_ty.into(), expected_ty.into(), &or_else).await
                    },
                )
                .await
            },
        )
    }

    #[track_caller]
    pub(super) fn spawn_require_numeric_type(&mut self, ty: SymTy<'db>, or_else: &dyn OrElse<'db>) {
        let or_else = or_else.to_arc();
        self.runtime.spawn(
            self,
            TaskDescription::RequireNumericType(ty),
            async move |env| require_numeric_type(env, ty, &or_else).await,
        )
    }

    #[track_caller]
    pub(super) fn spawn_require_future_type(
        &self,
        ty: SymTy<'db>,
        awaited_ty: SymTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) {
        let or_else = or_else.to_arc();
        self.runtime.spawn(
            self,
            TaskDescription::RequireFutureType(ty),
            async move |env| require_future_type(env, ty, awaited_ty, &or_else).await,
        )
    }

    /// Check whether any type in `tys` is known to be never (or error).
    /// If so, do nothing.
    /// Otherwise, if no type in `tys` is known to be never, invoke `op` (asynchronously).
    #[track_caller]
    pub fn spawn_if_not_never(
        &mut self,
        tys: &[SymTy<'db>],
        op: impl AsyncFnOnce(&mut Env<'db>) + 'db,
    ) {
        let _tys = tys.to_vec();
        self.runtime
            .spawn(self, TaskDescription::IfNotNever, async move |env| {
                // FIXME: check for never
                op(env).await
            })
    }

    pub fn describe_ty<'a, 'chk>(&'a self, ty: SymTy<'db>) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }

    #[track_caller]
    pub fn spawn<R>(
        &mut self,
        task_description: TaskDescription<'db>,
        op: impl AsyncFnOnce(&mut Self) -> R + 'db,
    ) where
        R: DeferResult,
    {
        self.runtime
            .spawn(self, task_description, async move |env| op(env).await)
    }

    pub(crate) fn require_expr_has_bool_ty(&mut self, expr: SymExpr<'db>) {
        let db = self.db();
        let boolean_ty = SymTy::boolean(db);
        self.spawn_require_assignable_type(expr.ty(db), boolean_ty, &BooleanTypeRequired { expr });
    }

    /// Check if the given (perm, type) variable is declared as leased.
    #[expect(dead_code)]
    pub fn is_leased_var(&self, _var: SymVariable<'db>) -> bool {
        false // FIXME
    }

    /// Span for code that prompted creation of inference variable `v`.
    pub(crate) fn infer_var_span(&self, v: InferVarIndex) -> Span<'db> {
        self.runtime.with_inference_var_data(v, |data| data.span())
    }

    /// Kind of this inference variable.
    pub(crate) fn infer_var_kind(&self, v: InferVarIndex) -> InferVarKind {
        self.runtime.with_inference_var_data(v, |data| data.kind())
    }

    pub(crate) fn variances(&self, n: SymTyName<'db>) -> Vec<Variance> {
        match n {
            SymTyName::Primitive(_) => vec![],
            SymTyName::Future => vec![Variance::Covariant],
            SymTyName::Tuple { arity } => vec![Variance::Covariant; arity],
            SymTyName::Aggregate(aggr) => aggr.variances(self.db()),
        }
    }

    /// If `infer` is a type variable, returns the permission variable associated with `infer`.
    /// If `infer` is a permission variable, just returns `infer`.
    pub fn perm_infer(&self, infer: InferVarIndex) -> InferVarIndex {
        self.runtime().perm_infer(infer)
    }

    #[track_caller]
    pub fn log(&mut self, message: &'static str, values: &[&dyn ToEventArgument<'db>]) {
        self.log.log(Location::caller(), message, values)
    }

    #[track_caller]
    pub fn indent<R>(
        &mut self,
        message: &'static str,
        values: &[&dyn ToEventArgument<'db>],
        op: impl AsyncFnOnce(&mut Self) -> R,
    ) -> impl Future<Output = R>
    where
        R: ToEventArgument<'db>,
    {
        let source_location = Location::caller();
        self.indent_with_source_location(source_location, message, values, op)
    }

    pub async fn indent_with_source_location<R>(
        &mut self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn ToEventArgument<'db>],
        op: impl AsyncFnOnce(&mut Self) -> R,
    ) -> R
    where
        R: ToEventArgument<'db>,
    {
        self.log.indent(source_location, message, values);
        let result = op(self).await;
        self.log.log(source_location, "result", &[&result]);
        self.log.undent(source_location, message);
        result
    }

    pub fn log_result<T>(&mut self, source_location: &'static Location<'static>, value: T) -> T
    where
        T: ToEventArgument<'db>,
    {
        self.log.log(source_location, "result", &[&value]);
        value
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

    async fn get(&self, env: &mut Env<'db>) -> SymTy<'db> {
        match self.state.get() {
            VariableType::Ast(ast_ty) => {
                self.state.set(VariableType::InProgress(ast_ty));
                let sym_ty = ast_ty.check_in_env(env).await;
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

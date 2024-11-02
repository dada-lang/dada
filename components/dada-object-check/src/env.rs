use std::sync::Arc;

use dada_ir_ast::{
    diagnostic::{Errors, Reported},
    span::Span,
};
use dada_ir_sym::{
    binder::BoundTerm,
    indices::InferVarIndex,
    scope::Scope,
    subst::SubstWith,
    symbol::{SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymPerm, SymTy},
};
use dada_util::Map;
use futures::{Stream, StreamExt};

use crate::{
    bound::{Bound, Direction, TransitiveBounds},
    check::Runtime,
    object_ir::{IntoObjectIr, ObjectGenericTerm, ObjectTy, ObjectTyKind},
    subobject::{require_assignable_object_type, require_numeric_type, require_sub_object_type},
    universe::Universe,
};

#[derive(Clone)]
pub struct Env<'db> {
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
    pub fn new(runtime: &Runtime<'db>, scope: Scope<'db, 'db>) -> Self {
        Self {
            universe: Universe::ROOT,
            runtime: runtime.clone(),
            scope: Arc::new(scope),
            variable_tys: Default::default(),
            variable_universes: Default::default(),
            return_ty: Default::default(),
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// Get the database
    pub fn db(&self) -> &'db dyn crate::Db {
        self.runtime.db
    }

    /// Open the given symbols as universally quantified.
    /// Creates a new universe.
    pub fn open_universally<T>(
        &mut self,
        runtime: &Runtime<'db>,
        variables: &[SymVariable<'db>],
        value: &T,
    ) -> T::LeafTerm
    where
        T: BoundTerm<'db>,
    {
        let db = runtime.db;

        match value.as_binder() {
            Err(leaf) => {
                return leaf.identity();
            }

            Ok(binder) => {
                self.increment_universe();
                Arc::make_mut(&mut self.variable_universes)
                    .extend(binder.variables.iter().map(|&v| (v, self.universe)));

                self.open_universally(runtime, variables, &binder.bound_value)
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
        self.fresh_ty_inference_var(span).into_object_ir(self.db())
    }

    pub fn fresh_perm_inference_var(&self, span: Span<'db>) -> SymPerm<'db> {
        self.fresh_inference_var(SymGenericKind::Type, span)
            .assert_perm(self.db())
    }

    pub fn require_assignable_object_type(
        &self,
        value_span: Span<'db>,
        value_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
        place_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = self.db();
        let value_ty = value_ty.into_object_ir(db);
        let place_ty = value_ty.into_object_ir(db);
        self.runtime.defer(self, value_span, move |env| async move {
            match require_assignable_object_type(&env, value_span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_sub_object_type(
        &self,
        span: Span<'db>,
        sub_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
        sup_ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = self.db();
        let value_ty = sub_ty.into_object_ir(db);
        let place_ty = value_ty.into_object_ir(db);
        self.runtime.defer(self, span, move |env| async move {
            match require_sub_object_type(&env, span, value_ty, place_ty).await {
                Ok(()) => (),
                Err(Reported(_)) => (),
            }
        })
    }

    pub fn require_numeric_type(
        &self,
        span: Span<'db>,
        ty: impl IntoObjectIr<'db, Object = ObjectTy<'db>>,
    ) {
        let db = self.db();
        let ty = ty.into_object_ir(db);
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
                            ObjectTyKind::Error(reported) => return,
                            ObjectTyKind::Infer(sym_infer_var_index) => continue 'next_bound,
                            ObjectTyKind::Named(..) | ObjectTyKind::Var(_) => continue 'next_ty,
                        }
                    }
                }

                op(env).await
            })
    }

    pub fn bound_inference_var(
        &self,
        infer_var: InferVarIndex,
        bound: Bound<impl Into<ObjectGenericTerm<'db>>>,
    ) -> Errors<()> {
        // FIXME
        Ok(())
    }

    pub fn transitive_lower_bounds(
        &self,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = ObjectTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::LowerBounds)
    }

    pub fn transitive_upper_bounds(
        &self,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = ObjectTy<'db>> + 'db {
        self.transitive_bounds(ty, Direction::UpperBounds)
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
}

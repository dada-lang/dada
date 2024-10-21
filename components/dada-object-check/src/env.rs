use std::sync::Arc;

use dada_ir_sym::{
    binder::Binder,
    scope::Scope,
    subst::Subst,
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{FromVar, SymGenericTerm, SymPerm, SymTy, SymTyKind, Var},
};
use dada_util::Map;
use futures::{Stream, StreamExt};
use salsa::Update;

use crate::{
    bound::{Bound, InferenceVarBounds},
    check::Check,
    object_ir::{IntoObjectIr, ObjectGenericTerm, ObjectTy, ObjectTyKind},
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
        binder: &Binder<T>,
    ) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>, Output = T> + Update,
    {
        let db = check.db;

        assert_eq!(variables.len(), binder.kinds.len());
        assert!(variables
            .iter()
            .zip(&binder.kinds)
            .all(|(s, &k)| s.kind(check.db) == k));

        self.increment_universe();
        Arc::make_mut(&mut self.variable_universes)
            .extend(variables.iter().map(|&v| (v, self.universe)));

        binder.open(db, |kind, sym_bound_var_index| {
            let symbol = variables[sym_bound_var_index.as_usize()];
            assert!(symbol.has_kind(db, kind));
            SymGenericTerm::var(db, kind, Var::Universal(symbol))
        })
    }

    /// Open the given symbols as existential inference variables
    /// in the current universe.
    pub fn open_existentially<T>(&self, check: &Check<'db>, binder: &Binder<T>) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>, Output = T> + Update,
    {
        let db = check.db;
        binder.open(db, |kind, sym_bound_var_index| {
            self.fresh_inference_var(check, kind)
        })
    }

    /// Create a substitution for `binder` consisting of inference variables
    pub fn existential_substitution<T: Update>(
        &self,
        check: &Check<'db>,
        binder: &Binder<T>,
    ) -> Vec<SymGenericTerm<'db>> {
        binder
            .kinds
            .iter()
            .map(|&kind| self.fresh_inference_var(check, kind))
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
    ) -> SymGenericTerm<'db> {
        check.fresh_inference_var(SymGenericKind::Perm, self.universe)
    }

    pub fn fresh_ty_inference_var(&self, check: &Check<'db>) -> SymTy<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type)
            .assert_type(check.db)
    }

    pub fn fresh_object_ty_inference_var(&self, check: &Check<'db>) -> ObjectTy<'db> {
        self.fresh_ty_inference_var(check).into_object_ir(check.db)
    }

    pub fn fresh_perm_inference_var(&self, check: &Check<'db>) -> SymPerm<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type)
            .assert_perm(check.db)
    }

    pub fn require_subobject(
        &self,
        check: &Check<'db>,
        sub: impl IntoObjectIr<'db>,
        sup: impl IntoObjectIr<'db>,
    ) {
        check.defer(self, |check, env| async move { todo!() });
    }

    pub fn bounds(
        &self,
        check: &Check<'db>,
        ty: SymTy<'db>,
    ) -> impl Stream<Item = Bound<SymTy<'db>>> + 'db {
        let db = check.db;
        if let &SymTyKind::Var(Var::Infer(inference_var)) = ty.kind(db) {
            <InferenceVarBounds<'db, SymGenericTerm<'db>>>::new(check, inference_var)
                .map(|b| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(Bound::LowerBound(ty))).boxed_local()
        }
    }

    pub fn object_bounds(
        &self,
        check: &Check<'db>,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = Bound<ObjectTy<'db>>> + 'db {
        let db = check.db;
        if let &ObjectTyKind::Var(Var::Infer(inference_var)) = ty.kind(db) {
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

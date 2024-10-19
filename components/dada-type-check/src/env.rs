use std::sync::Arc;

use dada_ir_sym::{
    scope::Scope,
    subst::Subst,
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{Binder, SymGenericTerm, SymPerm, SymTy, SymTyKind, Var},
};
use dada_util::Map;
use futures::{Stream, StreamExt};
use salsa::Update;

use crate::{
    bound::{Bound, InferenceVarBounds},
    checking_ir::{IntoObjectIr, ObjectGenericTerm, ObjectTy, ObjectTyKind},
    executor::Check,
    universe::Universe,
};

#[derive(Clone)]
pub struct Env<'db> {
    universe: Universe,

    /// Lexical scope for name resolution
    pub scope: Arc<Scope<'db, 'db>>,

    /// Free variables that are in scope as free universals.
    /// This includes generics but also place variables.
    /// The symbols are retained for better error messages.
    free_variables: Arc<Vec<SymVariable<'db>>>,

    /// Place variables that are in scope along with their types.
    variable_tys: Arc<Map<SymVariable<'db>, SymTy<'db>>>,

    /// If `None`, not type checking a function or method.
    return_ty: Option<SymTy<'db>>,
}

impl<'db> Env<'db> {
    /// Create an empty environment
    pub fn new(scope: Scope<'db, 'db>) -> Self {
        Self {
            universe: Universe::ROOT,
            scope: Arc::new(scope),
            variable_tys: Default::default(),
            free_variables: Default::default(),
            return_ty: Default::default(),
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// Opens two sets of binders at once where the symbols have been concatenated.
    /// Used for class members which are under the class / member binders.
    pub fn open_universally2<T>(
        &mut self,
        check: &Check<'_, 'db>,
        symbols: &[SymVariable<'db>],
        binder: Binder<Binder<T>>,
    ) -> T
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>, Output = T> + Update,
    {
        let (symbols1, symbols2) = symbols.split_at(binder.len());
        let b2 = self.open_universally(check, symbols1, binder);
        self.open_universally(check, symbols2, b2)
    }

    /// Open the given symbols as universally quantified.
    /// Creates a new universe.
    pub fn open_universally<T>(
        &mut self,
        check: &Check<'_, 'db>,
        symbols: &[SymVariable<'db>],
        binder: Binder<T>,
    ) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>, Output = T> + Update,
    {
        let db = check.db;

        assert_eq!(symbols.len(), binder.kinds.len());
        assert!(symbols
            .iter()
            .zip(&binder.kinds)
            .all(|(s, &k)| s.kind(check.db) == k));

        self.increment_universe();
        let base_index = self.free_variables.len();
        Arc::make_mut(&mut self.free_variables).extend(symbols);

        binder.open(db, |kind, sym_bound_var_index| {
            let symbol = symbols[sym_bound_var_index.as_usize()];
            assert!(symbol.has_kind(db, kind));
            SymGenericTerm::var(db, kind, Var::Universal(symbol))
        })
    }

    /// Open the given symbols as existential inference variables
    /// in the current universe.
    pub fn open_existentially<T>(&self, check: &Check<'_, 'db>, binder: &Binder<T>) -> T::Output
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
        check: &Check<'_, 'db>,
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
        check: &Check<'_, 'db>,
        kind: SymGenericKind,
    ) -> SymGenericTerm<'db> {
        check.fresh_inference_var(SymGenericKind::Perm, self.universe)
    }

    pub fn fresh_ty_inference_var(&self, check: &Check<'_, 'db>) -> SymTy<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type)
            .assert_type(check.db)
    }

    pub fn fresh_object_ty_inference_var(&self, check: &Check<'_, 'db>) -> ObjectTy<'db> {
        self.fresh_ty_inference_var(check).into_object_ir(check.db)
    }

    pub fn fresh_perm_inference_var(&self, check: &Check<'_, 'db>) -> SymPerm<'db> {
        self.fresh_inference_var(check, SymGenericKind::Type)
            .assert_perm(check.db)
    }

    pub fn require_subobject(
        &self,
        check: &Check<'_, 'db>,
        sub: impl IntoObjectIr<'db>,
        sup: impl IntoObjectIr<'db>,
    ) {
        check.defer(self, |check, env| async move { todo!() });
    }

    pub fn bounds<'chk>(
        &self,
        check: &Check<'chk, 'db>,
        ty: SymTy<'db>,
    ) -> impl Stream<Item = Bound<SymTy<'db>>> + 'chk {
        let db = check.db;
        if let &SymTyKind::Var(Var::Infer(inference_var)) = ty.kind(db) {
            <InferenceVarBounds<'_, '_, SymGenericTerm<'db>>>::new(check, inference_var)
                .map(|b| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(Bound::LowerBound(ty))).boxed_local()
        }
    }

    pub fn object_bounds<'chk>(
        &self,
        check: &Check<'chk, 'db>,
        ty: ObjectTy<'db>,
    ) -> impl Stream<Item = Bound<ObjectTy<'db>>> + 'chk {
        let db = check.db;
        if let &ObjectTyKind::Var(Var::Infer(inference_var)) = ty.kind(db) {
            <InferenceVarBounds<'_, '_, ObjectGenericTerm<'db>>>::new(check, inference_var)
                .map(|b| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(Bound::LowerBound(ty))).boxed_local()
        }
    }

    pub fn describe_ty<'a, 'chk>(
        &'a self,
        check: &'a Check<'chk, 'db>,
        ty: ObjectTy<'db>,
    ) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }
}

use std::sync::Arc;

use dada_ir_sym::{
    indices::SymVarIndex,
    scope::Scope,
    subst::Subst,
    symbol::{SymVariable, SymGenericKind, SymLocalVariable},
    ty::{Binder, GenericIndex, SymGenericTerm, SymPerm, SymTy, SymTyKind},
};
use dada_util::Map;
use futures::{Stream, StreamExt};
use salsa::Update;

use crate::{
    bound::{Bound, InferenceVarBounds},
    executor::Check,
    universe::Universe,
};

#[derive(Clone)]
pub struct Env<'db> {
    universe: Universe,

    /// Lexical scope for name resolution
    pub scope: Arc<Scope<'db, 'db>>,

    /// Generic variables that are in scope as free universals.
    /// The symbols are retained for better error messages.
    generic_variables: Arc<Vec<SymVariable<'db>>>,

    /// Local variables that are in scope along with their types.
    program_variables: Arc<Map<SymLocalVariable<'db>, SymTy<'db>>>,

    /// If `None`, not type checking a function or method.
    return_ty: Option<SymTy<'db>>,
}

impl<'db> Env<'db> {
    /// Create an empty environment
    pub fn new(scope: Scope<'db, 'db>) -> Self {
        Self {
            universe: Universe::ROOT,
            scope: Arc::new(scope),
            program_variables: Arc::new(Map::default()),
            generic_variables: Arc::new(Vec::new()),
            return_ty: None,
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// Opens two sets of binders at once where the symbols have been concatenated.
    /// Used for class members which are under the class / member binders.
    pub fn open_universally2<T: Subst<'db, Output = T> + Update>(
        &mut self,
        check: &Check<'_, 'db>,
        symbols: &[SymVariable<'db>],
        binder: Binder<Binder<T>>,
    ) -> T {
        let (symbols1, symbols2) = symbols.split_at(binder.len());
        let b2 = self.open_universally(check, symbols1, binder);
        self.open_universally(check, symbols2, b2)
    }

    /// Open the given symbols as universally quantified.
    /// Creates a new universe.
    pub fn open_universally<T: Subst<'db> + Update>(
        &mut self,
        check: &Check<'_, 'db>,
        symbols: &[SymVariable<'db>],
        binder: Binder<T>,
    ) -> T::Output {
        assert_eq!(symbols.len(), binder.kinds.len());
        assert!(symbols
            .iter()
            .zip(&binder.kinds)
            .all(|(s, &k)| s.kind(check.db) == k));

        self.increment_universe();
        let base_index = self.generic_variables.len();
        Arc::make_mut(&mut self.generic_variables).extend(symbols);

        binder.open(check.db, |kind, sym_bound_var_index| {
            let index = SymVarIndex::from(base_index + sym_bound_var_index.as_usize());
            SymGenericTerm::var(check.db, kind, GenericIndex::Universal(index))
        })
    }

    /// Open the given symbols as existential inference variables
    /// in the current universe.
    pub fn open_existentially<T: Subst<'db> + Update>(
        &self,
        check: &Check<'_, 'db>,
        binder: Binder<T>,
    ) -> T::Output {
        binder.open(check.db, |kind, sym_bound_var_index| {
            self.fresh_inference_var(check, kind)
        })
    }

    // Modify this environment to put it in a new universe.
    pub fn increment_universe(&mut self) {
        self.universe = self.universe.next();
    }

    /// Inserts a new program variable into the environment for later lookup.
    pub fn insert_program_variable(&mut self, lv: SymLocalVariable<'db>, ty: SymTy<'db>) {
        Arc::make_mut(&mut self.scope).push_link(lv);
        Arc::make_mut(&mut self.program_variables).insert(lv, ty);
    }

    /// Set the return type of the current function.
    pub fn set_return_ty(&mut self, ty: SymTy<'db>) {
        self.return_ty = Some(ty);
    }

    /// Returns the type of the given program variable.
    ///
    /// # Panics
    ///
    /// If the program variable is not present.
    pub fn program_variable_ty(&self, lv: SymLocalVariable<'db>) -> SymTy<'db> {
        self.program_variables.get(&lv).copied().unwrap()
    }

    pub fn fresh_inference_var(
        &self,
        check: &Check<'_, 'db>,
        kind: SymGenericKind,
    ) -> SymGenericTerm<'db> {
        check.fresh_inference_var(SymGenericKind::Perm, self.universe)
    }

    pub fn fresh_ty_inference_var(&self, check: &Check<'_, 'db>) -> SymTy<'db> {
        let SymGenericTerm::Type(ty) = self.fresh_inference_var(check, SymGenericKind::Type) else {
            unreachable!();
        };
        ty
    }

    pub fn fresh_perm_inference_var(&self, check: &Check<'_, 'db>) -> SymPerm<'db> {
        let SymGenericTerm::Perm(perm) = self.fresh_inference_var(check, SymGenericKind::Perm)
        else {
            unreachable!();
        };
        perm
    }

    pub fn require_subtype(&self, check: &Check<'_, 'db>, sub: SymTy<'db>, sup: SymTy<'db>) {
        check.defer(self, |check, env| async move { todo!() });
    }

    pub fn bounds<'chk>(
        &self,
        check: &Check<'chk, 'db>,
        ty: SymTy<'db>,
    ) -> impl Stream<Item = Bound<SymTy<'db>>> + 'chk {
        let db = check.db;
        if let &SymTyKind::Var(GenericIndex::Existential(inference_var)) = ty.kind(db) {
            InferenceVarBounds::new(check, inference_var)
                .map(|b| b.assert_type(db))
                .boxed_local()
        } else {
            futures::stream::once(futures::future::ready(Bound::LowerBound(ty))).boxed_local()
        }
    }

    pub fn describe_ty<'a, 'chk>(
        &'a self,
        check: &'a Check<'chk, 'db>,
        ty: SymTy<'db>,
    ) -> impl std::fmt::Display + 'a {
        format!("{ty:?}") // FIXME
    }
}

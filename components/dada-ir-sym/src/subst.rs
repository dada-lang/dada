use dada_ir_ast::diagnostic::Reported;
use dada_util::Map;
use salsa::Update;

use crate::{
    function::SymInputOutput,
    indices::{SymBinderIndex, SymBoundVarIndex},
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{
        Binder, SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymPlaceKind, SymTy, SymTyKind,
        SymTyName, Var,
    },
};

pub struct SubstitutionFns<'s, 'db, Term> {
    /// Invoked for variables bound in the [`INNERMOST`](`SymBinderIndex::INNERMOST`) binder
    /// when substitution begins. The result is automatically shifted with [`Subst::shift_into_binders`][]
    /// into any binders that we have traversed during the substitution.
    ///
    /// If this returns None, no substitution is performed.
    ///
    /// See [`Binder::open`][] for an example of this in use.
    pub bound_var: &'s mut dyn FnMut(SymGenericKind, SymBoundVarIndex) -> Option<Term>,

    /// Invoked for free variables.
    ///
    /// If this returns None, no substitution is performed.
    pub free_universal_var: &'s mut dyn FnMut(SymVariable<'db>) -> Option<Term>,

    /// Invoked to adjust the binder level for bound terms when:
    /// (a) the term is bound by some binder we have traversed or
    /// (b) the `bound_var` callback returns `None` for that term.
    ///
    /// See [`Binder::open`][] or [`Subst::shift_into_binders`][]
    /// for examples of this in use.
    pub binder_index: &'s mut dyn FnMut(SymBinderIndex) -> SymBinderIndex,
}

pub fn default_bound_var<Term>(_: SymGenericKind, _: SymBoundVarIndex) -> Option<Term> {
    None
}

pub fn default_free_var<'db, Term>(_: SymVariable<'db>) -> Option<Term> {
    None
}

pub fn default_binder_index(i: SymBinderIndex) -> SymBinderIndex {
    i
}

/// Core substitution trait: used to walk terms and produce new ones,
/// applying changes to the variables within.
pub trait Subst<'db> {
    /// The notion of a generic term appropriate for this type.
    type Term: Copy + HasKind<'db>;

    /// The type of the resulting term; typically `Self` but not always.
    type Output: Update;

    /// Reproduce `self` with no edits.
    fn identity(&self) -> Self::Output;

    /// Replace `self` applying the changes from `subst_fns`.
    ///
    /// # Parameters
    ///
    /// * `db`, the database
    /// * `start_binder`, the index of the binder we started from.
    ///   This always begins as `SymBinderIndex::INNERMOST`
    ///   but gets incremented as we traverse binders.
    /// * `subst_fns`, a struct containing callbacks for substitution
    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output;

    /// Convenient operation to shift all binder levels in `self` by `binders`.
    /// No-op if `binders` equals 0.
    ///
    /// See [`SymBinderIndex::shift_into_binders`][].
    fn shift_into_binders(&self, db: &'db dyn crate::Db, binders: usize) -> Self::Output {
        if binders == 0 {
            self.identity()
        } else {
            self.subst_with(
                db,
                SymBinderIndex::INNERMOST,
                &mut SubstitutionFns {
                    binder_index: &mut |b| b.shift_into_binders(binders),
                    free_universal_var: &mut default_free_var,
                    bound_var: &mut default_bound_var,
                },
            )
        }
    }

    /// Returns a version of `self` where universal free variables
    /// have been replaced by the corresponding entry in `terms`.
    /// If a variable is not present in `terms` it is not substituted.
    fn subst_vars(
        &self,
        db: &'db dyn crate::Db,
        map: &Map<SymVariable<'db>, Self::Term>,
    ) -> Self::Output {
        debug_assert!(map
            .iter()
            .all(|(&var, term)| term.has_kind(db, var.kind(db))));

        self.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                binder_index: &mut default_binder_index,
                bound_var: &mut default_bound_var,
                free_universal_var: &mut |var| map.get(&var).copied(),
            },
        )
    }

    /// Replace the variable `var` with `term`.
    fn subst_var(
        &self,
        db: &'db dyn crate::Db,
        var: SymVariable<'db>,
        term: Self::Term,
    ) -> Self::Output {
        debug_assert!(term.has_kind(db, var.kind(db)));

        self.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                binder_index: &mut default_binder_index,
                bound_var: &mut default_bound_var,
                free_universal_var: &mut |v| if v == var { Some(term) } else { None },
            },
        )
    }
}

impl<'db, T> Subst<'db> for &T
where
    T: Subst<'db>,
{
    type Term = T::Term;
    type Output = T::Output;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        T::subst_with(self, db, start_binder, subst_fns)
    }

    fn identity(&self) -> Self::Output {
        T::identity(self)
    }
}

impl<'db> Subst<'db> for SymGenericTerm<'db> {
    type Term = SymGenericTerm<'db>;
    type Output = Self;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        match self {
            SymGenericTerm::Type(ty) => {
                SymGenericTerm::Type(ty.subst_with(db, start_binder, subst_fns))
            }
            SymGenericTerm::Perm(perm) => {
                SymGenericTerm::Perm(perm.subst_with(db, start_binder, subst_fns))
            }
            SymGenericTerm::Place(place) => {
                SymGenericTerm::Place(place.subst_with(db, start_binder, subst_fns))
            }
            SymGenericTerm::Error(e) => {
                SymGenericTerm::Error(e.subst_with(db, start_binder, subst_fns))
            }
        }
    }

    fn identity(&self) -> Self::Output {
        *self
    }
}

impl<'db> Subst<'db> for Reported {
    type Output = Self;

    fn subst_with(
        &self,
        _db: &'db dyn crate::Db,
        _start_binder: SymBinderIndex,
        _subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        self.identity()
    }

    fn identity(&self) -> Self::Output {
        *self
    }
}

impl<'db> Subst<'db> for SymTy<'db> {
    type Output = Self;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymTyKind::Var(generic_index) => {
                subst_var(db, start_binder, subst_fns, self, *generic_index)
            }

            // Structucal cases
            SymTyKind::Perm(sym_perm, sym_ty) => SymTy::new(
                db,
                SymTyKind::Perm(
                    sym_perm.subst_with(db, start_binder, subst_fns),
                    sym_ty.subst_with(db, start_binder, subst_fns),
                ),
            ),
            SymTyKind::Named(sym_ty_name, vec) => SymTy::new(
                db,
                SymTyKind::Named(
                    sym_ty_name.subst_with(db, start_binder, subst_fns),
                    vec.iter()
                        .map(|g| g.subst_with(db, start_binder, subst_fns))
                        .collect(),
                ),
            ),
            SymTyKind::Unknown => self.identity(),
            SymTyKind::Error(reported) => SymTy::new(db, SymTyKind::Error(*reported)),
        }
    }

    fn identity(&self) -> Self::Output {
        *self
    }
}

impl<'db> Subst<'db> for SymPerm<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymPermKind::Var(generic_index) => {
                subst_var(db, start_binder, subst_fns, self, *generic_index)
            }

            // Structural cases
            SymPermKind::Shared(vec) => SymPerm::new(
                db,
                SymPermKind::Shared(
                    vec.iter()
                        .map(|g| g.subst_with(db, start_binder, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Leased(vec) => SymPerm::new(
                db,
                SymPermKind::Leased(
                    vec.iter()
                        .map(|g| g.subst_with(db, start_binder, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Given(vec) => SymPerm::new(
                db,
                SymPermKind::Given(
                    vec.iter()
                        .map(|g| g.subst_with(db, start_binder, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Error(reported) => SymPerm::new(
                db,
                SymPermKind::Error(reported.subst_with(db, start_binder, subst_fns)),
            ),
            SymPermKind::My => self.identity(),
            SymPermKind::Our => self.identity(),
        }
    }
}

impl<'db> Subst<'db> for SymTyName<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with(
        &self,
        _db: &'db dyn crate::Db,
        _start_binder: SymBinderIndex,
        _subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        self.identity()
    }
}

impl<'db> Subst<'db> for SymPlace<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymPlaceKind::Var(generic_index) => {
                subst_var(db, start_binder, subst_fns, self, generic_index)
            }

            // Structural cases
            SymPlaceKind::Field(sym_place, identifier) => SymPlace::new(
                db,
                SymPlaceKind::Field(
                    sym_place.subst_with(db, start_binder, subst_fns),
                    identifier,
                ),
            ),
            SymPlaceKind::Index(sym_place) => SymPlace::new(
                db,
                SymPlaceKind::Index(sym_place.subst_with(db, start_binder, subst_fns)),
            ),
            SymPlaceKind::Error(reported) => SymPlace::new(
                db,
                SymPlaceKind::Error(reported.subst_with(db, start_binder, subst_fns)),
            ),
        }
    }
}

impl<'db, T: Subst<'db> + Update> Subst<'db> for Binder<T> {
    type Output = Binder<T::Output>;

    fn identity(&self) -> Self::Output {
        Binder {
            kinds: self.kinds.clone(),
            bound_value: T::identity(&self.bound_value),
        }
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        let bound_value = self.bound_value.subst_with(db, start_binder + 1, subst_fns);
        Binder {
            kinds: self.kinds.clone(),
            bound_value,
        }
    }
}

impl<'db> Subst<'db> for SymInputOutput<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        self.clone()
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        SymInputOutput {
            input_tys: self.input_tys.subst_with(db, start_binder, subst_fns),
            output_ty: self.output_ty.subst_with(db, start_binder, subst_fns),
        }
    }
}

impl<'db, T> Subst<'db> for Vec<T>
where
    T: Subst<'db>,
{
    type Output = Vec<T::Output>;

    fn identity(&self) -> Self::Output {
        self.iter().map(T::identity).collect()
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    ) -> Self::Output {
        self.iter()
            .map(|t| t.subst_with(db, start_binder, subst_fns))
            .collect()
    }
}

fn subst_var<'db, Term>(
    db: &'db dyn crate::Db,
    start_binder: SymBinderIndex,
    subst_fns: &mut SubstitutionFns<'_, 'db, Self::Term>,
    term: &Term,
    generic_index: Var<'db>,
) -> Term
where
    Term: SubstGenericVar<'db>,
{
    match generic_index {
        Var::Bound(original_binder_index, sym_bound_var_index) => {
            let mut new_binder_index = || (subst_fns.binder_index)(original_binder_index);
            if original_binder_index == start_binder {
                match (subst_fns.bound_var)(SymGenericKind::Perm, sym_bound_var_index) {
                    Some(r) => {
                        Term::assert_kind(db, r).shift_into_binders(db, start_binder.as_usize())
                    }
                    None => Term::bound_var(db, new_binder_index(), sym_bound_var_index),
                }
            } else {
                Term::bound_var(db, new_binder_index(), sym_bound_var_index)
            }
        }
        Var::Universal(var) => match (subst_fns.free_universal_var)(var) {
            Some(r) => Term::assert_kind(db, r).shift_into_binders(db, start_binder.as_usize()),
            None => Term::identity(term),
        },
        Var::Infer(_) => term.identity(),
    }
}

trait SubstGenericVar<'db>: Subst<'db, Output = Self> {
    fn assert_kind(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self;

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self;
}

impl<'db> SubstGenericVar<'db> for SymPlace<'db> {
    fn assert_kind(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self {
        term.assert_place(db)
    }

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self {
        SymPlace::new(
            db,
            SymPlaceKind::Var(Var::Bound(binder_index, bound_var_index)),
        )
    }
}

impl<'db> SubstGenericVar<'db> for SymPerm<'db> {
    fn assert_kind(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self {
        term.assert_perm(db)
    }

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self {
        SymPerm::new(
            db,
            SymPermKind::Var(Var::Bound(binder_index, bound_var_index)),
        )
    }
}

impl<'db> SubstGenericVar<'db> for SymTy<'db> {
    fn assert_kind(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self {
        term.assert_type(db)
    }

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self {
        SymTy::new(
            db,
            SymTyKind::Var(Var::Bound(binder_index, bound_var_index)),
        )
    }
}

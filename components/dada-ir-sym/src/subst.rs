use std::fmt::Debug;

use dada_ir_ast::diagnostic::Reported;
use dada_util::Map;
use salsa::Update;

use crate::{
    binder::Binder,
    function::SymInputOutput,
    indices::{SymBinderIndex, SymBoundVarIndex},
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{
        FromVar, SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymPlaceKind, SymTy, SymTyKind,
        SymTyName, Var,
    },
};

pub struct SubstitutionFns<'s, 'db, Term> {
    /// Invoked for variables bound by some binder that has not been traversed during substitution;
    /// the binder index is relative to the start of the substitution.
    ///
    /// The result is automatically shifted with [`Subst::shift_into_binders`][] into any binders
    /// that we have traversed during the substitution.
    ///
    /// If this returns None, no substitution is performed.
    ///
    /// See [`Binder::open`][] for an example of this in use.
    pub free_bound_var:
        &'s mut dyn FnMut(SymGenericKind, SymBinderIndex, SymBoundVarIndex) -> Option<Term>,

    /// Invoked for free variables.
    ///
    /// If this returns None, no substitution is performed.
    pub free_universal_var: &'s mut dyn FnMut(SymVariable<'db>) -> Option<Term>,
}

pub fn default_free_bound_var<Term>(
    _: SymGenericKind,
    _: SymBinderIndex,
    _: SymBoundVarIndex,
) -> Option<Term> {
    None
}

pub fn default_free_universal_var<'db, Term>(_: SymVariable<'db>) -> Option<Term> {
    None
}

/// A type implemented by terms that can be substituted.
pub trait Subst<'db>: SubstWith<'db, Self::GenericTerm> + Debug {
    /// The notion of generic term appropriate for this type.
    /// When we substitute variables, this is the type of value that we replace them with.
    type GenericTerm: Copy + HasKind<'db> + Debug + FromVar<'db>;

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
                    free_universal_var: &mut default_free_universal_var,
                    free_bound_var: &mut |kind, binder_index, bound_var_index| {
                        Some(FromVar::var(
                            db,
                            kind,
                            Var::Bound(binder_index.shift_into_binders(binders), bound_var_index),
                        ))
                    },
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
        map: &Map<SymVariable<'db>, Self::GenericTerm>,
    ) -> Self::Output {
        debug_assert!(map
            .iter()
            .all(|(&var, term)| term.has_kind(db, var.kind(db))));

        self.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                free_bound_var: &mut default_free_bound_var,
                free_universal_var: &mut |var| map.get(&var).copied(),
            },
        )
    }

    /// Replace the variable `var` with `term`.
    fn subst_var(
        &self,
        db: &'db dyn crate::Db,
        var: SymVariable<'db>,
        term: Self::GenericTerm,
    ) -> Self::Output {
        debug_assert!(term.has_kind(db, var.kind(db)));

        self.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                free_bound_var: &mut default_free_bound_var,
                free_universal_var: &mut |v| if v == var { Some(term) } else { None },
            },
        )
    }
}

/// Core substitution operation: produce a version of this type
/// with variables replaced with instances of `Term`.
///
/// Most types implement this for only a single `Term`, but not all
/// (see the macro [`identity_subst`][]).
pub trait SubstWith<'db, Term> {
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
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output;
}

impl<'db, T> Subst<'db> for &T
where
    T: Subst<'db>,
{
    type GenericTerm = T::GenericTerm;
}

impl<'db, T, Term> SubstWith<'db, Term> for &T
where
    T: SubstWith<'db, Term>,
{
    type Output = T::Output;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        T::subst_with(self, db, start_binder, subst_fns)
    }

    fn identity(&self) -> Self::Output {
        T::identity(self)
    }
}

impl<'db> Subst<'db> for SymGenericTerm<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymGenericTerm<'db> {
    type Output = Self;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
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

impl<'db> Subst<'db> for SymTy<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymTy<'db> {
    type Output = Self;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
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

            SymTyKind::Never => self.identity(),
            SymTyKind::Error(_) => self.identity(),
            SymTyKind::Infer(_) => self.identity(),
        }
    }

    fn identity(&self) -> Self::Output {
        *self
    }
}

impl<'db> Subst<'db> for SymPerm<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymPerm<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
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
            SymPermKind::Infer(_) => self.identity(),
        }
    }
}

impl<'db> Subst<'db> for SymPlace<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymPlace<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
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
            SymPlaceKind::Infer(_) => self.identity(),
        }
    }
}

impl<'db, T: Subst<'db> + Update> Subst<'db> for Binder<T> {
    type GenericTerm = T::GenericTerm;
}

impl<'db, T: Subst<'db> + Update> SubstWith<'db, T::GenericTerm> for Binder<T> {
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
        subst_fns: &mut SubstitutionFns<'_, 'db, T::GenericTerm>,
    ) -> Self::Output {
        let bound_value = self.bound_value.subst_with(db, start_binder + 1, subst_fns);
        Binder {
            kinds: self.kinds.clone(),
            bound_value,
        }
    }
}

impl<'db> Subst<'db> for SymInputOutput<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymInputOutput<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        self.clone()
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
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
    type GenericTerm = T::GenericTerm;
}

impl<'db, T: Subst<'db>> SubstWith<'db, T::GenericTerm> for Vec<T> {
    type Output = Vec<T::Output>;

    fn identity(&self) -> Self::Output {
        self.iter().map(T::identity).collect()
    }

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db, T::GenericTerm>,
    ) -> Self::Output {
        self.iter()
            .map(|t| t.subst_with(db, start_binder, subst_fns))
            .collect()
    }
}

pub fn subst_var<'db, KTerm>(
    db: &'db dyn crate::Db,
    start_binder: SymBinderIndex,
    subst_fns: &mut SubstitutionFns<'_, 'db, KTerm::GenericTerm>,
    term: &KTerm,
    var: Var<'db>,
) -> KTerm
where
    KTerm: SubstGenericVar<'db>,
{
    match var {
        Var::Bound(binder_index, sym_bound_var_index) => {
            if binder_index >= start_binder {
                if let Some(r) = (subst_fns.free_bound_var)(
                    KTerm::KIND,
                    binder_index.shift_out_to(start_binder),
                    sym_bound_var_index,
                ) {
                    return KTerm::assert_kind(db, r)
                        .shift_into_binders(db, start_binder.as_usize());
                }
            }

            KTerm::bound_var(db, binder_index, sym_bound_var_index)
        }
        Var::Universal(var) => match (subst_fns.free_universal_var)(var) {
            Some(r) => KTerm::assert_kind(db, r).shift_into_binders(db, start_binder.as_usize()),
            None => KTerm::identity(term),
        },
    }
}

pub trait SubstGenericVar<'db>: Subst<'db, Output = Self> + Debug {
    const KIND: SymGenericKind;

    fn assert_kind(db: &'db dyn crate::Db, term: Self::GenericTerm) -> Self;

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self;
}

impl<'db> SubstGenericVar<'db> for SymPlace<'db> {
    const KIND: SymGenericKind = SymGenericKind::Place;

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
    const KIND: SymGenericKind = SymGenericKind::Perm;

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
    const KIND: SymGenericKind = SymGenericKind::Type;

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

/// For types that do not contain any potentially substitutable
/// content, we can use a simple impl. Note that while these types
/// default [`Subst::Term`][] type to `SymGenericTerm`,
/// that is just for convenience -- they implement [`SubstWith`][]
/// for any type `Term`.
macro_rules! identity_subst {
    (for $l:lifetime { $($t:ty,)* }) => {
        $(
            impl<$l> Subst<$l> for $t {
                type GenericTerm = SymGenericTerm<$l>;
            }

            impl<$l, Term> SubstWith<$l, Term> for $t {
                type Output = Self;

                fn identity(&self) -> Self::Output {
                    *self
                }

                fn subst_with(
                    &self,
                    _db: &$l dyn crate::Db,
                    _start_binder: SymBinderIndex,
                    _subst_fns: &mut SubstitutionFns<'_, $l, Term>,
                ) -> Self::Output {
                    *self
                }
            }
        )*
    };
}

identity_subst! {
    for 'db {
        (),
        Reported,
        SymGenericKind,
        SymTyName<'db>,
    }
}

use std::ops::Sub;

use dada_ir_ast::diagnostic::Reported;
use salsa::Update;

use crate::{
    function::SymInputOutput,
    indices::{SymBinderIndex, SymBoundVarIndex, SymVarIndex},
    symbol::{SymGenericKind, SymVariable},
    ty::{
        Binder, GenericIndex, SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymPlaceKind, SymTy,
        SymTyKind, SymTyName,
    },
};

pub struct SubstitutionFns<'s, 'db> {
    /// Invoked for variables bound in the [`INNERMOST`](`SymBinderIndex::INNERMOST`) binder
    /// when substitution begins. The result is automatically shifted with [`Subst::shift_into_binders`][]
    /// into any binders that we have traversed during the substitution.
    ///
    /// If this returns None, no substitution is performed.
    ///
    /// See [`Binder::open`][] for an example of this in use.
    pub bound_var:
        &'s mut dyn FnMut(SymGenericKind, SymBoundVarIndex) -> Option<SymGenericTerm<'db>>,

    /// Invoked for free variables.
    ///
    /// If this returns None, no substitution is performed.
    pub free_universal_var:
        &'s mut dyn FnMut(SymGenericKind, SymVarIndex) -> Option<SymGenericTerm<'db>>,

    /// Invoked to adjust the binder level for bound terms when:
    /// (a) the term is bound by some binder we have traversed or
    /// (b) the `bound_var` callback returns `None` for that term.
    ///
    /// See [`Binder::open`][] or [`Subst::shift_into_binders`][]
    /// for examples of this in use.
    pub binder_index: &'s mut dyn FnMut(SymBinderIndex) -> SymBinderIndex,
}

impl<'s, 'db> SubstitutionFns<'s, 'db> {
    pub fn default_bound_var(
        _: SymGenericKind,
        _: SymBoundVarIndex,
    ) -> Option<SymGenericTerm<'db>> {
        None
    }

    pub fn default_free_var(_: SymGenericKind, _: SymVarIndex) -> Option<SymGenericTerm<'db>> {
        None
    }

    pub fn default_binder_index(i: SymBinderIndex) -> SymBinderIndex {
        i
    }
}

pub trait Subst<'db> {
    type Output: Update;

    fn identity(&self) -> Self::Output;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output;

    fn shift_into_binders(&self, db: &'db dyn crate::Db, binders: SymBinderIndex) -> Self::Output {
        if binders == SymBinderIndex::INNERMOST {
            self.identity()
        } else {
            self.subst_with(
                db,
                binders,
                &mut SubstitutionFns {
                    binder_index: &mut |b| b.shift_into_binders(binders),
                    free_universal_var: &mut SubstitutionFns::default_free_var,
                    bound_var: &mut SubstitutionFns::default_bound_var,
                },
            )
        }
    }

    /// Returns a version of `self` where all (universal free variables
    /// have been replaced by the corresponding entry in `terms`.
    ///
    /// # Panics
    ///
    /// Panics if `self` contain any free variables with an index outside the range of `terms`
    /// or if the kind of a term does not match the kind of the free variable.
    fn subst_universal_free_vars(
        &self,
        db: &'db dyn crate::Db,
        mut terms: impl FnMut(SymVarIndex) -> Option<SymGenericTerm<'db>>,
    ) -> Self::Output {
        self.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                binder_index: &mut SubstitutionFns::default_binder_index,
                bound_var: &mut SubstitutionFns::default_bound_var,
                free_universal_var: &mut |var_kind, var_index| {
                    let Some(r) = terms(var_index) else {
                        return None;
                    };

                    assert!(r.has_kind(var_kind));

                    Some(r)
                },
            },
        )
    }
}

impl<'db, T> Subst<'db> for &T
where
    T: Subst<'db>,
{
    type Output = T::Output;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        T::subst_with(self, db, depth, subst_fns)
    }

    fn identity(&self) -> Self::Output {
        T::identity(self)
    }
}

impl<'db> Subst<'db> for SymGenericTerm<'db> {
    type Output = Self;

    fn subst_with(
        &self,
        db: &'db dyn crate::Db,
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        match self {
            SymGenericTerm::Type(ty) => SymGenericTerm::Type(ty.subst_with(db, depth, subst_fns)),
            SymGenericTerm::Perm(perm) => {
                SymGenericTerm::Perm(perm.subst_with(db, depth, subst_fns))
            }
            SymGenericTerm::Place(place) => {
                SymGenericTerm::Place(place.subst_with(db, depth, subst_fns))
            }
            SymGenericTerm::Error(e) => SymGenericTerm::Error(e.subst_with(db, depth, subst_fns)),
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
        _depth: SymBinderIndex,
        _subst_fns: &mut SubstitutionFns<'_, 'db>,
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymTyKind::Var(generic_index) => subst_var(db, depth, subst_fns, self, *generic_index),

            // Structucal cases
            SymTyKind::Perm(sym_perm, sym_ty) => SymTy::new(
                db,
                SymTyKind::Perm(
                    sym_perm.subst_with(db, depth, subst_fns),
                    sym_ty.subst_with(db, depth, subst_fns),
                ),
            ),
            SymTyKind::Named(sym_ty_name, vec) => SymTy::new(
                db,
                SymTyKind::Named(
                    sym_ty_name.subst_with(db, depth, subst_fns),
                    vec.iter()
                        .map(|g| g.subst_with(db, depth, subst_fns))
                        .collect(),
                ),
            ),
            SymTyKind::Unknown => self.identity(),
            SymTyKind::Error(reported) => SymTy::new(
                db,
                SymTyKind::Error(reported.subst_with(db, depth, subst_fns)),
            ),
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymPermKind::Var(generic_index) => {
                subst_var(db, depth, subst_fns, self, *generic_index)
            }

            // Structural cases
            SymPermKind::Shared(vec) => SymPerm::new(
                db,
                SymPermKind::Shared(
                    vec.iter()
                        .map(|g| g.subst_with(db, depth, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Leased(vec) => SymPerm::new(
                db,
                SymPermKind::Leased(
                    vec.iter()
                        .map(|g| g.subst_with(db, depth, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Given(vec) => SymPerm::new(
                db,
                SymPermKind::Given(
                    vec.iter()
                        .map(|g| g.subst_with(db, depth, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Error(reported) => SymPerm::new(
                db,
                SymPermKind::Error(reported.subst_with(db, depth, subst_fns)),
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
        _depth: SymBinderIndex,
        _subst_fns: &mut SubstitutionFns<'_, 'db>,
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymPlaceKind::Var(generic_index) => {
                subst_var(db, depth, subst_fns, self, generic_index)
            }

            // Structural cases
            SymPlaceKind::Field(sym_place, identifier) => SymPlace::new(
                db,
                SymPlaceKind::Field(sym_place.subst_with(db, depth, subst_fns), identifier),
            ),
            SymPlaceKind::Index(sym_place) => SymPlace::new(
                db,
                SymPlaceKind::Index(sym_place.subst_with(db, depth, subst_fns)),
            ),
            SymPlaceKind::Error(reported) => SymPlace::new(
                db,
                SymPlaceKind::Error(reported.subst_with(db, depth, subst_fns)),
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        let bound_value = self.bound_value.subst_with(db, depth + 1, subst_fns);
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        SymInputOutput {
            input_tys: self.input_tys.subst_with(db, depth, subst_fns),
            output_ty: self.output_ty.subst_with(db, depth, subst_fns),
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
        depth: SymBinderIndex,
        subst_fns: &mut SubstitutionFns<'_, 'db>,
    ) -> Self::Output {
        self.iter()
            .map(|t| t.subst_with(db, depth, subst_fns))
            .collect()
    }
}

fn subst_var<'db, Term>(
    db: &'db dyn crate::Db,
    depth: SymBinderIndex,
    subst_fns: &mut SubstitutionFns<'_, 'db>,
    term: &Term,
    generic_index: GenericIndex,
) -> Term
where
    Term: SubstGenericVar<'db>,
{
    match generic_index {
        GenericIndex::Bound(original_binder_index, sym_bound_var_index) => {
            let mut new_binder_index = || (subst_fns.binder_index)(original_binder_index);
            if original_binder_index == depth {
                match (subst_fns.bound_var)(SymGenericKind::Perm, sym_bound_var_index) {
                    Some(r) => Term::assert_kind(db, r).shift_into_binders(db, depth),
                    None => Term::bound_var(db, new_binder_index(), sym_bound_var_index),
                }
            } else {
                Term::bound_var(db, new_binder_index(), sym_bound_var_index)
            }
        }
        GenericIndex::Universal(var_index) => {
            match (subst_fns.free_universal_var)(SymGenericKind::Perm, var_index) {
                Some(r) => Term::assert_kind(db, r).shift_into_binders(db, depth),
                None => Term::identity(term),
            }
        }
        GenericIndex::Existential(_) => term.identity(),
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
            SymPlaceKind::Var(GenericIndex::Bound(binder_index, bound_var_index)),
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
            SymPermKind::Var(GenericIndex::Bound(binder_index, bound_var_index)),
        )
    }
}

impl<'db> SubstGenericVar<'db> for SymTy<'db> {
    fn assert_kind(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self {
        term.assert_ty(db)
    }

    fn bound_var(
        db: &'db dyn crate::Db,
        binder_index: SymBinderIndex,
        bound_var_index: SymBoundVarIndex,
    ) -> Self {
        SymTy::new(
            db,
            SymTyKind::Var(GenericIndex::Bound(binder_index, bound_var_index)),
        )
    }
}

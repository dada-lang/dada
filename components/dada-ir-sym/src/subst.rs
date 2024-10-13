use dada_ir_ast::diagnostic::Reported;
use salsa::Update;

use crate::{
    function::SymInputOutput,
    indices::{SymBinderIndex, SymBoundVarIndex},
    symbol::{SymGenericKind, SymLocalVariable},
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
    /// See [`Binder::open`][] for an example of this in use.
    pub bound_var:
        &'s mut dyn FnMut(SymGenericKind, SymBoundVarIndex) -> Option<SymGenericTerm<'db>>,

    /// Invoked to adjust the binder level for bound terms when:
    /// (a) the term is bound by some binder we have traversed or
    /// (b) the `bound_var` callback returns `None` for that term.
    ///
    /// See [`Binder::open`][] or [`Subst::shift_into_binders`][]
    /// for examples of this in use.
    pub binder_index: &'s mut dyn FnMut(SymBinderIndex) -> SymBinderIndex,

    /// Invoked to adjust the local variable that begins a [`SymPlace`][].
    pub local_var: &'s mut dyn FnMut(SymLocalVariable<'db>) -> Option<SymPlace<'db>>,
}

impl<'s, 'db> SubstitutionFns<'s, 'db> {
    pub fn default_bound_var(
        _: SymGenericKind,
        _: SymBoundVarIndex,
    ) -> Option<SymGenericTerm<'db>> {
        None
    }

    pub fn default_binder_index(i: SymBinderIndex) -> SymBinderIndex {
        i
    }

    pub fn default_local_var(_: SymLocalVariable<'db>) -> Option<SymPlace<'db>> {
        None
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
                    bound_var: &mut SubstitutionFns::default_bound_var,
                    local_var: &mut SubstitutionFns::default_local_var,
                },
            )
        }
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
            // Interesting case
            SymTyKind::Var(generic_index) => match generic_index {
                GenericIndex::Bound(sym_binder_index, sym_bound_var_index) => {
                    if sym_binder_index == depth {
                        match (subst_fns.bound_var)(SymGenericKind::Type, sym_bound_var_index) {
                            Some(r) => r.assert_type(db).shift_into_binders(db, depth),
                            None => SymTy::new(
                                db,
                                SymTyKind::Var(GenericIndex::Bound(
                                    (subst_fns.binder_index)(sym_binder_index),
                                    sym_bound_var_index,
                                )),
                            ),
                        }
                    } else {
                        SymTy::new(
                            db,
                            SymTyKind::Var(GenericIndex::Bound(
                                (subst_fns.binder_index)(sym_binder_index),
                                sym_bound_var_index,
                            )),
                        )
                    }
                }
                GenericIndex::Universal(_) => self.identity(),
                GenericIndex::Existential(_) => self.identity(),
            },

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
            SymPermKind::Var(generic_index) => match generic_index {
                GenericIndex::Bound(sym_binder_index, sym_bound_var_index) => {
                    let sym_binder_index = (subst_fns.binder_index)(sym_binder_index);
                    if sym_binder_index == depth {
                        match (subst_fns.bound_var)(SymGenericKind::Perm, sym_bound_var_index) {
                            Some(r) => r.assert_perm(db).shift_into_binders(db, depth),
                            None => SymPerm::new(
                                db,
                                SymPermKind::Var(GenericIndex::Bound(
                                    sym_binder_index,
                                    sym_bound_var_index,
                                )),
                            ),
                        }
                    } else {
                        SymPerm::new(
                            db,
                            SymPermKind::Var(GenericIndex::Bound(
                                sym_binder_index,
                                sym_bound_var_index,
                            )),
                        )
                    }
                }
                GenericIndex::Universal(_) | GenericIndex::Existential(_) => self.identity(),
            },

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
            SymPlaceKind::LocalVariable(lv) => match (subst_fns.local_var)(lv) {
                Some(r) => r,
                None => self.identity(),
            },

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

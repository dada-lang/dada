use std::fmt::Debug;

use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_util::{Map, Never};

use crate::{
    ir::binder::{Binder, BoundTerm, NeverBinder},
    ir::functions::SymInputOutput,
    ir::types::{
        AssertKind, HasKind, SymGenericKind, SymGenericTerm, SymPerm, SymPermKind, SymPlace,
        SymPlaceKind, SymTy, SymTyKind, SymTyName,
    },
    ir::variables::{FromVar, SymVariable},
};

use super::{classes::SymField, functions::SymFunction, indices::InferVarIndex};

pub struct SubstitutionFns<'s, 'db, Term> {
    /// Invoked for free variables.
    ///
    /// If this returns None, no substitution is performed.
    pub free_var: &'s mut dyn FnMut(SymVariable<'db>) -> Option<Term>,

    /// Invoked for inference variables.
    ///
    /// If this returns None, no substitution is performed.
    pub infer_var: &'s mut dyn FnMut(InferVarIndex) -> Option<Term>,
}

pub fn default_free_var<Term>(_: SymVariable<'_>) -> Option<Term> {
    None
}

/// A type implemented by terms that can be substituted.
pub trait Subst<'db>: SubstWith<'db, Self::GenericTerm> + Debug {
    /// The notion of generic term appropriate for this type.
    /// When we substitute variables, this is the type of value that we replace them with.
    type GenericTerm: Copy + HasKind<'db> + Debug + FromVar<'db>;

    /// Returns a version of `self` where universal free variables
    /// have been replaced by the corresponding entry in `terms`.
    /// If a variable is not present in `terms` it is not substituted.
    fn subst_vars(
        &self,
        db: &'db dyn crate::Db,
        map: &Map<SymVariable<'db>, Self::GenericTerm>,
    ) -> Self::Output {
        debug_assert!(
            map.iter()
                .all(|(&var, term)| term.has_kind(db, var.kind(db)))
        );

        self.subst_with(
            db,
            &mut Default::default(),
            &mut SubstitutionFns {
                free_var: &mut |var| map.get(&var).copied(),
                infer_var: &mut |_| None,
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
            &mut Default::default(),
            &mut SubstitutionFns {
                free_var: &mut |v| if v == var { Some(term) } else { None },
                infer_var: &mut |_| None,
            },
        )
    }

    /// Replace all inference variables with whatever is returned by `op`;
    /// if `op` returns None, the inference variable is left unchanged.
    fn resolve_infer_var(
        &self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        mut op: impl FnMut(InferVarIndex) -> Option<Self::GenericTerm>,
    ) -> Self::Output {
        self.subst_with(
            db,
            bound_vars,
            &mut SubstitutionFns {
                free_var: &mut |_| None,
                infer_var: &mut op,
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
    type Output;

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
    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output;
}

impl<'db> Subst<'db> for Never {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db, Term> SubstWith<'db, Term> for Never {
    type Output = Never;

    fn identity(&self) -> Self::Output {
        unreachable!()
    }

    fn subst_with<'subst>(
        &'subst self,
        _db: &'db dyn crate::Db,
        _bound_vars: &mut Vec<SymVariable<'db>>,
        _subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        unreachable!()
    }
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        T::subst_with(self, db, bound_vars, subst_fns)
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match self {
            SymGenericTerm::Type(ty) => {
                SymGenericTerm::Type(ty.subst_with(db, bound_vars, subst_fns))
            }
            SymGenericTerm::Perm(perm) => {
                SymGenericTerm::Perm(perm.subst_with(db, bound_vars, subst_fns))
            }
            SymGenericTerm::Place(place) => {
                SymGenericTerm::Place(place.subst_with(db, bound_vars, subst_fns))
            }
            SymGenericTerm::Error(e) => {
                SymGenericTerm::Error(e.subst_with(db, bound_vars, subst_fns))
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymTyKind::Var(var) => subst_var(db, bound_vars, subst_fns, *var),
            SymTyKind::Infer(v) => {
                if let Some(term) = (subst_fns.infer_var)(*v) {
                    term.assert_type(db)
                } else {
                    self.identity()
                }
            }

            // Structucal cases
            SymTyKind::Perm(sym_perm, sym_ty) => SymTy::new(
                db,
                SymTyKind::Perm(
                    sym_perm.subst_with(db, bound_vars, subst_fns),
                    sym_ty.subst_with(db, bound_vars, subst_fns),
                ),
            ),
            SymTyKind::Named(sym_ty_name, vec) => SymTy::new(
                db,
                SymTyKind::Named(
                    sym_ty_name.subst_with(db, bound_vars, subst_fns),
                    vec.iter()
                        .map(|g| g.subst_with(db, bound_vars, subst_fns))
                        .collect(),
                ),
            ),
            SymTyKind::Never => self.identity(),
            SymTyKind::Error(_) => self.identity(),
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

    fn subst_with<'subst>(
        &self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match self.kind(db) {
            // Variables
            SymPermKind::Var(var) => subst_var(db, bound_vars, subst_fns, *var),
            SymPermKind::Infer(v) => {
                if let Some(term) = (subst_fns.infer_var)(*v) {
                    term.assert_perm(db)
                } else {
                    self.identity()
                }
            }

            // Structural cases
            SymPermKind::Shared(vec) => SymPerm::new(
                db,
                SymPermKind::Shared(
                    vec.iter()
                        .map(|g| g.subst_with(db, bound_vars, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Leased(vec) => SymPerm::new(
                db,
                SymPermKind::Leased(
                    vec.iter()
                        .map(|g| g.subst_with(db, bound_vars, subst_fns))
                        .collect(),
                ),
            ),
            SymPermKind::Error(reported) => SymPerm::new(
                db,
                SymPermKind::Error(reported.subst_with(db, bound_vars, subst_fns)),
            ),
            SymPermKind::My => self.identity(),
            SymPermKind::Our => self.identity(),
            SymPermKind::Apply(left, right) => SymPerm::new(
                db,
                SymPermKind::Apply(
                    left.subst_with(db, bound_vars, subst_fns),
                    right.subst_with(db, bound_vars, subst_fns),
                ),
            ),
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match self.kind(db) {
            SymPlaceKind::Var(var) => subst_var(db, bound_vars, subst_fns, *var),
            SymPlaceKind::Field(sym_place, identifier) => SymPlace::new(
                db,
                SymPlaceKind::Field(sym_place.subst_with(db, bound_vars, subst_fns), *identifier),
            ),
            SymPlaceKind::Index(sym_place) => SymPlace::new(
                db,
                SymPlaceKind::Index(sym_place.subst_with(db, bound_vars, subst_fns)),
            ),
            SymPlaceKind::Error(reported) => SymPlace::new(
                db,
                SymPlaceKind::Error(reported.subst_with(db, bound_vars, subst_fns)),
            ),
            SymPlaceKind::Erased => SymPlace::new(db, SymPlaceKind::Erased),
        }
    }
}

impl<'db, T: BoundTerm<'db>> Subst<'db> for Binder<'db, T>
where
    T::Output: BoundTerm<'db>,
{
    type GenericTerm = T::GenericTerm;
}

impl<'db, T: BoundTerm<'db>> SubstWith<'db, T::GenericTerm> for Binder<'db, T> {
    type Output = Binder<'db, T>;

    fn identity(&self) -> Self::Output {
        Binder {
            variables: self.variables.clone(),
            bound_value: T::identity(&self.bound_value),
        }
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, T::GenericTerm>,
    ) -> Self::Output {
        let len = bound_vars.len();
        bound_vars.extend_from_slice(&self.variables);
        let bound_value = self.bound_value.subst_with(db, bound_vars, subst_fns);
        bound_vars.truncate(len);

        Binder {
            variables: self.variables.clone(),
            bound_value,
        }
    }
}

impl<'db, T> Subst<'db> for NeverBinder<T>
where
    T: Debug,
{
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db, T, Term> SubstWith<'db, Term> for NeverBinder<T> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        unreachable!()
    }

    fn subst_with<'subst>(
        &'subst self,
        _db: &'db dyn crate::Db,
        _bound_vars: &mut Vec<SymVariable<'db>>,
        _subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        unreachable!()
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        SymInputOutput {
            input_tys: self.input_tys.subst_with(db, bound_vars, subst_fns),
            output_ty: self.output_ty.subst_with(db, bound_vars, subst_fns),
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

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, T::GenericTerm>,
    ) -> Self::Output {
        self.iter()
            .map(|t| t.subst_with(db, bound_vars, subst_fns))
            .collect()
    }
}

pub fn subst_var<'db, Output, Term>(
    db: &'db dyn crate::Db,
    bound_vars: &mut Vec<SymVariable<'db>>,
    subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    var: SymVariable<'db>,
) -> Output
where
    Term: AssertKind<'db, Output>,
    Output: FromVar<'db>,
{
    let var_appears_free = !bound_vars.contains(&var);

    if var_appears_free {
        if let Some(term) = (subst_fns.free_var)(var) {
            return term.assert_kind(db);
        }
    }

    Output::var(db, var)
}

/// For types that do not contain any potentially substitutable
/// content, we can use a simple impl. Note that while these types
/// default [`ir::subst::Term`][] type to `SymGenericTerm`,
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

                fn subst_with<'subst>(
                    &self,
                    _db: &$l dyn crate::Db,
                    _bound_vars: &mut  Vec<SymVariable<'db>>,
                    _subst_fns: &mut SubstitutionFns<'_, $l, Term>,
                ) -> Self::Output {
                    *self
                }
            }
        )*
    };
}
pub(crate) use identity_subst; // Now classic paths Just Work™

identity_subst! {
    for 'db {
        (),
        Reported,
        SymGenericKind,
        SymTyName<'db>,
        Span<'db>,
        SymFunction<'db>,
        SymField<'db>,
    }
}

impl<'db, Term, T> SubstWith<'db, Term> for Option<T>
where
    T: SubstWith<'db, Term>,
{
    type Output = Option<T::Output>;

    fn identity(&self) -> Self::Output {
        self.as_ref().map(|v| v.identity())
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        self.as_ref()
            .map(|v| v.subst_with(db, bound_vars, subst_fns))
    }
}

impl<'db, O, E, Term> SubstWith<'db, Term> for Result<O, E>
where
    O: SubstWith<'db, Term>,
    E: SubstWith<'db, Term>,
{
    type Output = Result<O::Output, E::Output>;

    fn identity(&self) -> Self::Output {
        match self {
            Ok(v) => Ok(v.identity()),
            Err(e) => Err(e.identity()),
        }
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, Term>,
    ) -> Self::Output {
        match self {
            Ok(v) => Ok(v.subst_with(db, bound_vars, subst_fns)),
            Err(e) => Err(e.subst_with(db, bound_vars, subst_fns)),
        }
    }
}

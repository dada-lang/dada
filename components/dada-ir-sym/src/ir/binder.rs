use std::fmt::Debug;

use dada_util::Never;
use salsa::Update;

use crate::{
    ir::subst::{Subst, SubstitutionFns},
    ir::symbol::SymVariable,
    ir::types::{HasKind, SymGenericKind},
};

/// Indicates a binder for generic variables
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct Binder<'db, T: BoundTerm<'db>> {
    pub variables: Vec<SymVariable<'db>>,
    pub bound_value: T,
}

impl<'db, T: BoundTerm<'db>> Binder<'db, T> {
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn kind(&self, db: &'db dyn crate::Db, index: usize) -> SymGenericKind {
        self.variables[index].kind(db)
    }

    /// Generic way to "open" a binder, giving a function that computes the replacement
    /// value for each bound variable. You may preference [`Self::substitute`][] for the
    /// most common cases.
    ///
    /// # Parameters
    ///
    /// * `db`, the database
    /// * `func`, compute the replacement for bound variable at the given index
    pub fn open(
        &self,
        db: &'db dyn crate::Db,
        mut func: impl FnMut(usize) -> T::GenericTerm,
    ) -> T::Output
    where
        T: Subst<'db>,
    {
        let mut cache = vec![None; self.len()];

        self.bound_value.subst_with(
            db,
            &mut Default::default(),
            &mut SubstitutionFns {
                free_var: &mut |var| {
                    if let Some(index) = self.variables.iter().position(|v| *v == var) {
                        Some(*cache[index].get_or_insert_with(|| func(index)))
                    } else {
                        None
                    }
                },
            },
        )
    }

    /// Open the binder by replacing each variable with the corresponding term from `substitution`.
    ///
    /// # Panics
    ///
    /// If `substitution` does not have the correct length or there is a kind-mismatch.
    pub fn substitute(
        &self,
        db: &'db dyn crate::Db,
        substitution: &[impl Into<T::GenericTerm> + Copy],
    ) -> T::Output {
        assert_eq!(self.len(), substitution.len());
        self.open(db, |index| {
            let term = substitution[index].into();
            assert!(term.has_kind(db, self.kind(db, index)));
            term
        })
    }

    /// Maps the bound contents to something else
    /// using the contents of argument term `arg`.
    ///
    /// `arg` will automatically have any bound variables
    /// shifted by 1 to account for having been inserted
    /// into a new binder.
    ///
    /// If no arg is needed just supply `()`.
    ///
    /// NB. The argument is a `fn` to prevent accidentally leaking context.
    pub fn map<U>(self, _db: &'db dyn crate::Db, op: impl FnOnce(T) -> U) -> Binder<'db, U>
    where
        U: BoundTerm<'db>,
    {
        Binder {
            variables: self.variables,
            bound_value: op(self.bound_value),
        }
    }

    /// Maps the bound contents to something else
    /// using the contents of argument term `arg`.
    ///
    /// `arg` will automatically have any bound variables
    /// shifted by 1 to account for having been inserted
    /// into a new binder.
    ///
    /// If no arg is needed just supply `()`.
    ///
    /// NB. The argument is a `fn` to prevent accidentally leaking context.
    pub fn map_ref<U>(&self, _db: &'db dyn crate::Db, op: impl FnOnce(&T) -> U) -> Binder<'db, U>
    where
        U: BoundTerm<'db>,
    {
        Binder {
            variables: self.variables.clone(),
            bound_value: op(&self.bound_value),
        }
    }
}

impl<'db, T> std::ops::Index<usize> for Binder<'db, T>
where
    T: BoundTerm<'db>,
{
    type Output = SymVariable<'db>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.variables[index]
    }
}

/// A value that can appear in a binder
pub trait BoundTerm<'db>: Update + Subst<'db, Output = Self> + Sized {
    const BINDER_LEVELS: usize;
    type BoundTerm: BoundTerm<'db, LeafTerm = Self::LeafTerm>;
    type LeafTerm: Subst<'db, Output = Self::LeafTerm>;

    fn bind(
        db: &'db dyn crate::Db,
        symbols_to_bind: &mut dyn Iterator<Item = Vec<SymVariable<'db>>>,
        leaf_value: Self::LeafTerm,
    ) -> Self;

    fn as_binder(&self) -> Result<&Binder<'db, Self::BoundTerm>, &Self::LeafTerm>;
}

pub trait LeafBoundTerm<'db>: Update + Subst<'db, Output = Self> {}

impl<'db, T> BoundTerm<'db> for T
where
    T: LeafBoundTerm<'db>,
{
    const BINDER_LEVELS: usize = 0;
    type BoundTerm = NeverBinder<T>;
    type LeafTerm = T;

    fn bind(
        _db: &'db dyn crate::Db,
        symbols_to_bind: &mut dyn Iterator<Item = Vec<SymVariable<'db>>>,
        value: T,
    ) -> Self {
        assert!(
            symbols_to_bind.next().is_none(),
            "incorrect number of binding levels in iterator"
        );
        value
    }

    fn as_binder(&self) -> Result<&Binder<'db, NeverBinder<Self>>, &Self> {
        Err(self)
    }
}

impl<'db> LeafBoundTerm<'db> for Never {}

impl<'db, T> BoundTerm<'db> for Binder<'db, T>
where
    T: BoundTerm<'db>,
{
    const BINDER_LEVELS: usize = T::BINDER_LEVELS + 1;
    type BoundTerm = T;
    type LeafTerm = T::LeafTerm;

    fn bind(
        db: &'db dyn crate::Db,
        symbols_to_bind: &mut dyn Iterator<Item = Vec<SymVariable<'db>>>,
        leaf_value: T::LeafTerm,
    ) -> Self {
        // Extract next level of bound symbols for use in this binder;
        // if this unwrap fails, user gave wrong number of `Binder<_>` types
        // for the scope.
        let variables = symbols_to_bind.next().unwrap();

        // Introduce whatever binders are needed to go from the innermost
        // value type `T` to `U`.
        let u = T::bind(db, symbols_to_bind, leaf_value);
        Binder {
            variables,
            bound_value: u,
        }
    }

    fn as_binder(&self) -> Result<&Binder<'db, T>, &Self::LeafTerm> {
        Ok(self)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NeverBinder<T> {
    _data: Never,
    _value: T,
}

unsafe impl<T> Update for NeverBinder<T> {
    unsafe fn maybe_update(_old_pointer: *mut Self, _new_value: Self) -> bool {
        unreachable!()
    }
}

impl<'db, T: Subst<'db, Output = T>> BoundTerm<'db> for NeverBinder<T> {
    const BINDER_LEVELS: usize = 0;

    type BoundTerm = NeverBinder<T>;

    type LeafTerm = T;

    fn bind(
        _db: &'db dyn crate::Db,
        _symbols_to_bind: &mut dyn Iterator<Item = Vec<SymVariable<'db>>>,
        _leaf_value: Self::LeafTerm,
    ) -> Self {
        unreachable!()
    }

    fn as_binder(&self) -> Result<&Binder<'db, Self::BoundTerm>, &Self::LeafTerm> {
        unreachable!()
    }
}

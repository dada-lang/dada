use salsa::Update;

use crate::{
    indices::{SymBinderIndex, SymBoundVarIndex},
    subst::{self, Subst, SubstitutionFns},
    symbol::{HasKind, SymGenericKind},
};

/// Indicates a binder for generic variables
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct Binder<T: Update> {
    /// Number of bound generic terms
    pub kinds: Vec<SymGenericKind>,

    pub bound_value: T,
}

impl<T: Update> Binder<T> {
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    /// Generic way to "open" a binder, giving a function that computes the replacement
    /// value for each bound variable. You may preference [`Self::substitute`][] for the
    /// most common cases.
    pub fn open<'db>(
        &self,
        db: &'db dyn crate::Db,
        mut func: impl FnMut(SymGenericKind, SymBoundVarIndex) -> T::GenericTerm,
    ) -> T::Output
    where
        T: Subst<'db>,
    {
        let mut cache = vec![None; self.kinds.len()];

        self.bound_value.subst_with(
            db,
            SymBinderIndex::INNERMOST,
            &mut SubstitutionFns {
                free_bound_var: &mut |kind, binder_index, bound_var_index| {
                    // We don't expect to invoke `open` for some inner binder that still contains references
                    // to things bound in outer binders.
                    assert_eq!(binder_index, SymBinderIndex::INNERMOST);

                    Some(*cache[bound_var_index.as_usize()].get_or_insert_with(|| {
                        assert_eq!(kind, self.kinds[bound_var_index.as_usize()]);
                        func(kind, bound_var_index)
                    }))
                },
                free_universal_var: &mut subst::default_free_universal_var,
            },
        )
    }

    /// Open the binder by replacing each variable with the corresponding term from `substitution`.
    ///
    /// # Panics
    ///
    /// If `substitution` does not have the correct length or there is a kind-mismatch.
    pub fn substitute<'db>(
        &self,
        db: &'db dyn crate::Db,
        substitution: &[impl Into<T::GenericTerm> + Copy],
    ) -> T::Output
    where
        T: Subst<'db>,
    {
        assert_eq!(self.len(), substitution.len());
        self.open(db, |kind, index| {
            let term = substitution[index.as_usize()].into();
            assert!(term.has_kind(db, kind));
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
    pub fn map<'db, U, A>(
        self,
        db: &'db dyn crate::Db,
        arg: A,
        op: fn(&'db dyn crate::Db, T, A::Output) -> U,
    ) -> Binder<U>
    where
        U: Update,
        A: Subst<'db>,
    {
        Binder {
            kinds: self.kinds,
            bound_value: op(db, self.bound_value, arg.shift_into_binders(db, 1)),
        }
    }
}

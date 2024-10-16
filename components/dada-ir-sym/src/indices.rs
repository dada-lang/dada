use salsa::Update;

/// Also known as a "de Bruijn index", a binder index
/// Identifies the binder in which a bound a variable is bound.
/// Counts outward from the innermost binder, so 0 indicates
/// the innermost binder, 1 the binder around that, and so forth.
///
/// Example:
///
/// ```dada
/// class Vec[type A] { // <-- binder
///     fn find(self, value: type B: Comparable[A]) -> bool { // <-- binder
///         ... // (Note: in the body, A and B "appear free".)
///     }
/// }
/// ```
///
/// Inside of the signature of `find`, references to `A` have binder index 1
/// and references to `B` have binder index 0.
///
/// Binder indices are used in external facing signatures where
/// variables will be substituted with another value when used.
/// Inside of function bodies, or in contexts where we are checking
/// generically against all types, bound variables will be represented
/// as "free variables", most often free *universal* ("∀") variables.
/// See e.g. [`SymUniversalVarIndex`][].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymBinderIndex(usize);

impl SymBinderIndex {
    pub const INNERMOST: SymBinderIndex = SymBinderIndex(0);

    pub fn shift_into_binders(self, binders: SymBinderIndex) -> Self {
        SymBinderIndex(self.0 + binders.0)
    }

    pub fn shift_out(self) -> Self {
        SymBinderIndex(self.0.checked_sub(1).unwrap())
    }
}

impl From<usize> for SymBinderIndex {
    fn from(value: usize) -> Self {
        SymBinderIndex(value)
    }
}

impl std::ops::Add<usize> for SymBinderIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        SymBinderIndex(self.0.checked_add(rhs).unwrap())
    }
}

/// Identifies a particular variable within a binder.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymBoundVarIndex(usize);

impl SymBoundVarIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for SymBoundVarIndex {
    fn from(value: usize) -> Self {
        SymBoundVarIndex(value)
    }
}

/// Identifies a particular free variable.
/// Indices are assigned with `0` representing the "outermost" free variable.
///
/// # Example
///
/// ```dada
/// class Vec[type A] { // <-- binder
///     fn find(self, value: type B: Comparable[A]) -> bool { // <-- binder
///         ... // (Note: in the body, A and B "appear free".)
///     }
/// }
/// ```
///
/// Inside the function body, `A` has index 0, `B` has index 1.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymVarIndex(usize);

impl SymVarIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for SymVarIndex {
    fn from(value: usize) -> Self {
        SymVarIndex(value)
    }
}

impl std::ops::Add<usize> for SymVarIndex {
    type Output = SymVarIndex;

    fn add(self, value: usize) -> Self {
        Self::from(self.as_usize().checked_add(value).unwrap())
    }
}

impl std::ops::Sub<SymVarIndex> for SymVarIndex {
    type Output = usize;

    fn sub(self, value: SymVarIndex) -> usize {
        self.as_usize().checked_sub(value.as_usize()).unwrap()
    }
}

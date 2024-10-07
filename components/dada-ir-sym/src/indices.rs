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
}

impl From<usize> for SymBinderIndex {
    fn from(value: usize) -> Self {
        SymBinderIndex(value)
    }
}

/// Identifies a particular variable within a binder.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymBoundVarIndex(usize);

impl From<usize> for SymBoundVarIndex {
    fn from(value: usize) -> Self {
        SymBoundVarIndex(value)
    }
}

/// Identifies a particular universal ("∀") variable.
/// Indices are assigned with `0` representing the "outermost" bound variable.
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

impl From<usize> for SymVarIndex {
    fn from(value: usize) -> Self {
        SymVarIndex(value)
    }
}

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
    /// The innermost binder starts with index 0.
    pub const INNERMOST: SymBinderIndex = SymBinderIndex(0);

    /// Get the binder level as an integer, with 0 indicating the innermost binder.
    pub fn as_usize(self) -> usize {
        self.0
    }

    /// Shifting *into* a binder means incrementing the index.
    /// Consider
    ///
    /// ```dada
    /// class Foo[type A] { // <-- binder for class
    ///     let field: A;
    ///     fn bar(self) { // <-- binder for method
    ///     }
    /// }
    /// ```
    ///
    /// If we want to refer to `A` inside the field, it has binder depth 0,
    /// as the class is the innermost body.
    ///
    /// But if we want to "shift" a reference to `A` so its valid inside the method,
    /// we have to increment the index to 1, to account for the method's binder.
    pub fn shift_into_binders(self, binders: usize) -> Self {
        SymBinderIndex(self.0 + binders)
    }

    /// Shift in by 1 binding level.
    /// See [`Self::shift_into_binders`][] for an example.
    pub fn shift_in(self) -> Self {
        self.shift_into_binders(1)
    }

    /// Shifting out is the inverse of shifting in.
    /// See [`Self::shift_into_binders`][] for an example.
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

/// Identifies a particular inference variable during type checking.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymInferVarIndex(usize);

impl SymInferVarIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for SymInferVarIndex {
    fn from(value: usize) -> Self {
        SymInferVarIndex(value)
    }
}

impl std::ops::Add<usize> for SymInferVarIndex {
    type Output = SymInferVarIndex;

    fn add(self, value: usize) -> Self {
        Self::from(self.as_usize().checked_add(value).unwrap())
    }
}

impl std::ops::Sub<SymInferVarIndex> for SymInferVarIndex {
    type Output = usize;

    fn sub(self, value: SymInferVarIndex) -> usize {
        self.as_usize().checked_sub(value.as_usize()).unwrap()
    }
}

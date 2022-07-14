use crate::span::FileSpan;

salsa::entity2! {
    /// A "spanned spanned" is a `Specifier` that carries a span for diagnostics.
    entity SpannedSpecifier in crate::Jar {
        #[id] specifier: Specifier,

        /// If true, the specifier was not explicitly given by the user
        /// but was defaulted.
        defaulted: bool,

        /// Span of the specifier keywords, or storage name if specified was
        /// defaulted.
        span: FileSpan,
    }
}

impl SpannedSpecifier {
    /// Creates a new `SpannedSpecifier` for a variable/field that didn't
    /// have an explicit specifier.
    pub fn new_defaulted(db: &dyn crate::Db, name_span: FileSpan) -> Self {
        Self::new(db, Specifier::Any, true, name_span)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug)]
pub enum Specifier {
    Leased,
    Shleased,
    Any,
}

impl Specifier {
    /// True if values stored under this specifier must be uniquely
    /// accessible (my, leased) and cannot be jointly accessible (our, shleased).
    ///
    /// [`Specifier::Any`] returns false.
    pub fn must_be_unique(self) -> bool {
        match self {
            Specifier::Leased => true,
            Specifier::Shleased => false,
            Specifier::Any => false,
        }
    }

    /// True if values stored under this specifier must be owned (my, our)
    /// and cannot be leased (leased, shleased).
    ///
    /// [`Specifier::Any`] returns false.
    pub fn must_be_owned(self) -> bool {
        match self {
            Specifier::Shleased | Specifier::Leased => false,
            Specifier::Any => false,
        }
    }
}

impl std::fmt::Display for Specifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Specifier::Shleased => write!(f, "shleased"),
            Specifier::Leased => write!(f, "leased"),
            Specifier::Any => write!(f, "any"),
        }
    }
}

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the atomic properties for the various storage modes,
/// and we want that to be atomic if any step was atomic.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Atomic {
    No,
    Yes,
}

impl std::ops::BitOr for Atomic {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Atomic {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the joint properties for the various storage modes,
/// and we want that to be atomic if any step was joint.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Joint {
    No,
    Yes,
}

impl std::ops::BitOr for Joint {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Joint {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the owned properties for the various storage modes,
/// and we want that to be atomic if any step was joint.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Leased {
    No,
    Yes,
}

impl std::ops::BitOr for Leased {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Leased {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}

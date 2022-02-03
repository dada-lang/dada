use crate::kw::Keyword;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum StorageMode {
    Shared,
    Var,
    Atomic,
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

impl StorageMode {
    pub fn keyword(self) -> Keyword {
        match self {
            StorageMode::Shared => Keyword::Shared,
            StorageMode::Var => Keyword::Var,
            StorageMode::Atomic => Keyword::Atomic,
        }
    }

    /// Is this storage mode atomic?
    pub fn atomic(self) -> Atomic {
        match self {
            StorageMode::Atomic => Atomic::Yes,
            StorageMode::Shared | StorageMode::Var => Atomic::No,
        }
    }

    /// Is this storage mode joint, meaning that it does not
    /// grant exclusive access?
    pub fn joint(self) -> Joint {
        match self {
            StorageMode::Shared => Joint::Yes,
            StorageMode::Atomic | StorageMode::Var => Joint::No,
        }
    }
}

impl std::fmt::Display for StorageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.keyword(), f)
    }
}

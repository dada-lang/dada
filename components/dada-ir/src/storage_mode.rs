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

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the joint properties for the various storage modes,
/// and we want that to be atomic if any step was joint.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Joint {
    No,
    Yes,
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

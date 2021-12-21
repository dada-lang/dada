use crate::kw::Keyword;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum StorageMode {
    Shared,
    Var,
    Atomic,
}

impl StorageMode {
    pub fn keyword(self) -> Keyword {
        match self {
            StorageMode::Shared => Keyword::Shared,
            StorageMode::Var => Keyword::Var,
            StorageMode::Atomic => Keyword::Atomic,
        }
    }
}

impl std::fmt::Display for StorageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.keyword(), f)
    }
}

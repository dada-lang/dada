//! String literals in Dada are actually kind of complex.
//! They can include expressions and so forth.

use crate::{token_tree::TokenTree, word::Word};

#[salsa::interned]
#[customize(DebugWithDb)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::len_without_is_empty)]
pub struct FormatString {
    pub len: u32,

    /// List of sections from a string like `"foo{bar}baz" -- that example would
    /// have three sections.
    pub sections: Vec<FormatStringSection>,
}

impl FormatString {
    /// True if the format string is empty.
    pub fn is_empty(self, db: &dyn crate::Db) -> bool {
        self.len(db) == 0
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for FormatString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        salsa::DebugWithDb::fmt(&self.sections(db), f, db)
    }
}

#[salsa::interned]
#[customize(DebugWithDb)]
pub struct FormatStringSection {
    pub data: FormatStringSectionData,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FormatStringSectionData {
    /// Plain text to be emitted directly.
    Text(Word),

    /// A token tree for an expression.
    TokenTree(TokenTree),
}

impl FormatStringSection {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self, db: &dyn crate::Db) -> u32 {
        match self.data(db) {
            FormatStringSectionData::Text(w) => w.len(db),
            FormatStringSectionData::TokenTree(tree) => tree.len(db),
        }
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for FormatStringSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        salsa::DebugWithDb::fmt(&self.data(db), f, db)
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for FormatStringSectionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self {
            FormatStringSectionData::Text(word) => {
                f.debug_tuple("Text").field(&word.debug(db)).finish()
            }
            FormatStringSectionData::TokenTree(tree) => {
                f.debug_tuple("TokenTree").field(&tree.debug(db)).finish()
            }
        }
    }
}

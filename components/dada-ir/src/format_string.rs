//! String literals in Dada are actually kind of complex.
//! They can include expressions and so forth.

use crate::{token_tree::TokenTree, word::Word};

#[salsa::interned(FormatString in crate::Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatStringData {
    pub len: u32,

    /// List of sections from a string like `"foo{bar}baz" -- that example would
    /// have three sections.
    pub sections: Vec<FormatStringSection>,
}

impl FormatString {
    pub fn len(&self, db: &dyn crate::Db) -> u32 {
        self.data(db).len
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for FormatString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        salsa::DebugWithDb::fmt(self.data(db), f, db)
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for FormatStringData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        salsa::DebugWithDb::fmt(&self.sections, f, db)
    }
}

#[salsa::interned(FormatStringSection in crate::Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FormatStringSectionData {
    /// Plain text to be emitted directly.
    Text(Word),

    /// A token tree for an expression.
    TokenTree(TokenTree),
}

impl FormatStringSection {
    pub fn len(&self, db: &dyn crate::Db) -> u32 {
        match self.data(db) {
            FormatStringSectionData::Text(w) => w.len(db),
            FormatStringSectionData::TokenTree(tree) => tree.len(db),
        }
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for FormatStringSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        salsa::DebugWithDb::fmt(self.data(db), f, db)
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for FormatStringSectionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
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

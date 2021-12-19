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

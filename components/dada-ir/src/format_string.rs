//! String literals in Dada are actually kind of complex.
//! They can include expressions and so forth.

use crate::{token_tree::TokenTree, word::Word};

#[salsa::interned(FormatString in crate::Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatStringData {
    pub sections: Vec<FormatStringSection>,
}

#[salsa::interned(FormatStringSection in crate::Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FormatStringSectionData {
    Text(Word),
    TokenTree(TokenTree),
}

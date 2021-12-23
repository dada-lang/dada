use crate::{filename::Filename, token_tree::TokenTree};

/// "Code" represents a block of code attached to a method.
/// After parsing, it just contains a token tree, but you can...
///
/// * use the `ast` method from the `dada_parse` prelude to
///   parse it into an `Ast`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Code(TokenTree);

impl Code {
    pub fn new(token_tree: TokenTree) -> Self {
        Self(token_tree)
    }

    pub fn token_tree(self) -> TokenTree {
        self.0
    }

    pub fn filename(self, db: &dyn crate::Db) -> Filename {
        self.token_tree().filename(db)
    }
}

pub mod syntax;
pub mod validated;

use crate::token_tree::TokenTree;

salsa::entity2! {
    /// "Code" represents a block of code attached to a method.
    /// After parsing, it just contains a token tree, but you can...
    ///
    /// * use the `ast` method from the `dada_parse` prelude to
    ///   parse it into an `Ast`.
    entity Code in crate::Jar {
        tokens: TokenTree,
    }
}

pub mod syntax;

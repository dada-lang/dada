use crate::parser::Parser;

use dada_ir::{parameter::Parameter, token_tree::TokenTree};

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_parameters(db: &dyn crate::Db, token_tree: TokenTree) -> Vec<Parameter> {
    Parser::new(db, token_tree).parse_only_parameters()
}

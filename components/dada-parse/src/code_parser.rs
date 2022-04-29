use crate::parser::Parser;

use dada_ir::code::{syntax::Tree, Code};
use dada_ir::token_tree::TokenTree;

#[salsa::memoized(in crate::Jar)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> Tree {
    let body = code.body_tokens;
    Parser::new(db, body).parse_code_body(code)
}

#[salsa::memoized(in crate::Jar)]
pub fn parse_repl_expr(db: &dyn crate::Db, token_tree: TokenTree) -> Option<Tree> {
    Parser::new(db, token_tree).parse_repl_expr(token_tree)
}

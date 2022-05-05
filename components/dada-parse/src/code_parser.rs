use crate::parser::Parser;

use dada_ir::code::{syntax::Tree, UnparsedCode};

#[salsa::memoized(in crate::Jar)]
pub fn parse_code(db: &dyn crate::Db, code: UnparsedCode) -> Tree {
    let body = code.body_tokens;
    Parser::new(db, body).parse_code_body(code)
}

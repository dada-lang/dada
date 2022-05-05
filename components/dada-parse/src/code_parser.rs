use crate::{parser::Parser, prelude::*};

use dada_ir::{code::syntax::Tree, function::Function};

#[salsa::memoized(in crate::Jar)]
pub fn parse_function_body(db: &dyn crate::Db, function: Function) -> Tree {
    let body = function.unparsed_code(db).body_tokens;
    Parser::new(db, body).parse_code_body(function.parameters(db))
}

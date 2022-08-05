use crate::{parser::Parser, prelude::*};

use dada_ir::{code::syntax::Tree, function::Function};
use salsa::DebugWithDb;

#[salsa::tracked(specify)]
pub fn parse_function_body(db: &dyn crate::Db, function: Function) -> Tree {
    if let Some(unparsed_code) = function.unparsed_code(db) {
        let body = unparsed_code.body_tokens;
        Parser::new(db, body).parse_code_body(function.parameters(db))
    } else {
        panic!(
            "cannot parse function `{:?}` which did not have unparsed code",
            function.debug(db)
        );
    }
}

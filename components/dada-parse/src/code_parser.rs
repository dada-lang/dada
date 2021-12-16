use crate::parser::Parser;

use dada_ir::{
    code::{Ast, Code},
    diagnostic::Diagnostic,
};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> (Ast, Vec<Diagnostic>) {
    let token_tree = code.tokens(db);
    let mut errors = vec![];
    let mut parser = Parser::new(db, token_tree, &mut errors);
    let result = parser.parse_ast();
    (result, errors)
}

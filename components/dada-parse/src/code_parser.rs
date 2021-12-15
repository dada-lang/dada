use crate::{parser::Parser, tokens::Tokens};

use dada_ir::{
    code::{Ast, Code},
    diagnostic::Diagnostic,
};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> (Ast, Vec<Diagnostic>) {
    let token_tree = code.tokens(db);
    let tokens = Tokens::new(db, token_tree);
    let mut errors = vec![];
    let mut parser = Parser::new(db, tokens, &mut errors);
    let result = parser.parse_ast();
    (result, errors)
}

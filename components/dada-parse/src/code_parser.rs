use crate::parser::Parser;

use dada_ir::code::{Ast, Code, Spans};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> Ast {
    let token_tree = code.tokens(db);
    Parser::new(db, token_tree).parse_ast()
}

#[salsa::memoized(in crate::Jar ref)]
pub fn spans_for_parsed_code(db: &dyn crate::Db, code: Code) -> Spans {
    let token_tree = code.tokens(db);
    Parser::new(db, token_tree).parse_ast_and_spans().1
}

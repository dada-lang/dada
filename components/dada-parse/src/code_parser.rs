use crate::parser::Parser;

use dada_ir::code::{
    syntax::{Spans, Tree},
    Code,
};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> Tree {
    let token_tree = code.token_tree();
    Parser::new(db, token_tree).parse_syntax_tree()
}

#[salsa::memoized(in crate::Jar ref)]
pub fn spans_for_parsed_code(db: &dyn crate::Db, code: Code) -> Spans {
    let token_tree = code.token_tree();
    Parser::new(db, token_tree).parse_syntax_tree_and_spans().1
}

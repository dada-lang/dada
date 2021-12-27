use crate::parser::Parser;

use dada_ir::code::{syntax::Tree, Code};

#[salsa::memoized(in crate::Jar)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> Tree {
    let token_tree = code.token_tree();
    Parser::new(db, token_tree).parse_syntax_tree(code)
}

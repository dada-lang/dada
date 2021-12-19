use crate::parser::Parser;

use dada_ir::code::{Ast, Code};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> Ast {
    let token_tree = code.tokens(db);
    Parser::new(db, token_tree).parse_ast()
}

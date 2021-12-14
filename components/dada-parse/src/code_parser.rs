use crate::{parser::Parser, tokens::Tokens};

use dada_ir::{diagnostic::Diagnostic, item::Item, word::Word};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_code(db: &dyn crate::Db, code: Code) -> (Ast, Vec<Diagnostic>) {
    let token_tree = code.tokens(db);
    let tokens = Tokens::new(db, token_tree);
    let mut parser = Parser::new(db, filename, tokens);
    let result = parser.parse_items();
    (result, parser.into_errors())
}

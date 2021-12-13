use crate::{parser::Parser, tokens::Tokens};

use dada_ir::{diagnostic::Diagnostic, item::Item, word::Word};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_file(db: &dyn crate::Db, filename: Word) -> (Vec<Item>, Vec<Diagnostic>) {
    let token_tree = dada_lex::lex_file(db, filename);
    let tokens = Tokens::new(db, token_tree);
    let mut parser = Parser::new(db, filename, tokens);
    let result = parser.parse_items();
    (result, parser.into_errors())
}

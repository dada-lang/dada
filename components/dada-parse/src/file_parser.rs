use crate::parser::Parser;

use dada_ir::{diagnostic::Diagnostic, item::Item, word::Word};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_file(db: &dyn crate::Db, filename: Word) -> (Vec<Item>, Vec<Diagnostic>) {
    let token_tree = dada_lex::lex_file(db, filename);
    let mut errors = vec![];
    let mut parser = Parser::new(db, token_tree, &mut errors);
    let result = parser.parse_items();
    (result, errors)
}

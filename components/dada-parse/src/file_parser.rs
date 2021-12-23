use crate::parser::Parser;

use dada_ir::{filename::Filename, item::Item};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_file(db: &dyn crate::Db, filename: Filename) -> Vec<Item> {
    let token_tree = dada_lex::lex_file(db, filename);
    let mut parser = Parser::new(db, token_tree);
    parser.parse_items()
}

use crate::parser::Parser;

use dada_ir::{filename::Filename, source_file::SourceFile};

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_file(db: &dyn crate::Db, filename: Filename) -> SourceFile {
    let token_tree = dada_lex::lex_file(db, filename);
    let mut parser = Parser::new(db, token_tree);
    parser.parse_source_file()
}

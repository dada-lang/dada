use crate::parser::Parser;

use dada_ir::{input_file::InputFile, source_file::SourceFile};

#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_file(db: &dyn crate::Db, input_file: InputFile) -> SourceFile {
    let token_tree = dada_lex::lex_file(db, input_file);
    let mut parser = Parser::new(db, token_tree);
    parser.parse_source_file()
}

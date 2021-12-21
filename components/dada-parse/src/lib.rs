#![feature(trait_upcasting)]
#![feature(let_else)]
#![allow(incomplete_features)]

mod code_parser;
mod file_parser;
mod parser;
mod token_test;
mod tokens;

#[salsa::jar(Db)]
pub struct Jar(
    code_parser::parse_code,
    code_parser::spans_for_parsed_code,
    file_parser::parse_file,
);

pub trait Db: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}

pub use code_parser::parse_code;
pub use code_parser::spans_for_parsed_code;
pub use file_parser::parse_file;
pub mod prelude;

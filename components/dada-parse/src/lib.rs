#![feature(trait_upcasting)]
#![feature(let_else)]
#![allow(incomplete_features)]

pub mod code_parser;
mod file_parser;
mod parameter_parser;
mod parser;
mod token_test;
mod tokens;

#[salsa::jar(Db)]
pub struct Jar(
    code_parser::parse_code,
    code_parser::parse_repl_expr,
    file_parser::parse_file,
    parameter_parser::parse_parameters,
);

pub trait Db: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}

pub mod prelude;

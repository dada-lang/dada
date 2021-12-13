#![feature(trait_upcasting)]
#![feature(let_else)]
#![allow(incomplete_features)]

mod parse;
mod token_test;
mod tokens;

#[salsa::jar(Db)]
pub struct Jar(parse::parse_file);

pub trait Db: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_lex::Db + dada_ir::Db {}

pub use parse::parse_file;

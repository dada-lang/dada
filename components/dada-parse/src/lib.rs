use dada_ir::Ir;
use dada_lex::Lexer;

#[salsa::jar(Parser)]
pub struct Jar;

pub trait Parser: salsa::DbWithJar<Jar> + Lexer + Ir {}
impl<T> Parser for T where T: salsa::DbWithJar<Jar> + Lexer + Ir {}

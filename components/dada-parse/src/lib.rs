use dada_ir::Ast;
use dada_lex::Lexer;

#[salsa::jar(Parser)]
pub struct Jar;

pub trait Parser: salsa::DbWithJar<Jar> + Lexer + Ast {}
impl<T> Parser for T where T: salsa::DbWithJar<Jar> + Lexer + Ast {}

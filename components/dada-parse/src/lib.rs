use dada_lex::Lexer;
use std::iter::Peekable;

#[salsa::jar(Parser)]
pub struct Jar;

pub trait Parser: salsa::DbWithJar<Jar> + Lexer {}
impl<T> Parser for T where T: salsa::DbWithJar<Jar> + Lexer {}

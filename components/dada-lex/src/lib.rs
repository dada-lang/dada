#![feature(trait_upcasting)]
#![allow(incomplete_features)]

mod lex;
mod lines;

#[salsa::jar(Db)]
pub struct Jar(lex::lex_file, lines::line_table);

pub trait Db: salsa::DbWithJar<Jar> + dada_manifest::Db + dada_ir::Db {
    fn lex(&self) -> &dyn Db;
}
impl<T> Db for T
where
    T: salsa::DbWithJar<Jar> + dada_manifest::Db + dada_ir::Db,
{
    fn lex(&self) -> &dyn Db {
        self
    }
}

pub use lex::closing_delimiter;
pub use lex::lex_file;
pub use lines::line_column;

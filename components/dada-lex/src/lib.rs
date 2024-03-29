#![feature(trait_upcasting)]
#![allow(incomplete_features)]

mod lex;
pub mod prelude;

#[salsa::jar(db = Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db {
    fn lex(&self) -> &dyn Db;
}
impl<T> Db for T
where
    T: salsa::DbWithJar<Jar> + dada_ir::Db,
{
    fn lex(&self) -> &dyn Db {
        self
    }
}

pub use lex::closing_delimiter;
pub use lex::lex_file;

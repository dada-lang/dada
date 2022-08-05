#![allow(incomplete_features)]
#![feature(trait_upcasting)]

mod check;

#[salsa::jar(db = Db)]
pub struct Jar(check::check_input_file);

pub trait Db:
    salsa::DbWithJar<Jar>
    + dada_brew::Db
    + dada_ir::Db
    + dada_lex::Db
    + dada_parse::Db
    + dada_validate::Db
{
}

impl<T> Db for T where
    T: salsa::DbWithJar<Jar>
        + dada_brew::Db
        + dada_ir::Db
        + dada_lex::Db
        + dada_parse::Db
        + dada_validate::Db
{
}

pub use check::check_input_file;

#![feature(let_else)]
#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

#[salsa::jar(db = Db)]
pub struct Jar(ext::class_field_names);

pub trait Db:
    salsa::DbWithJar<Jar>
    + dada_ir::Db
    + dada_parse::Db
    + dada_brew::Db
    + dada_error_format::Db
    + dada_validate::Db
{
}

impl<T> Db for T where
    T: salsa::DbWithJar<Jar>
        + dada_ir::Db
        + dada_parse::Db
        + dada_brew::Db
        + dada_error_format::Db
        + dada_validate::Db
{
}

mod error;
mod ext;
pub mod heap_graph;
pub mod kernel;
pub mod machine;
mod moment;
mod run;
mod step;
mod thunk;

pub use error::DiagnosticError;
pub use run::interpret;

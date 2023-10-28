//! "Brews" the bir (i.e., compiles)

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

#[salsa::jar(db = Db)]
pub struct Jar(brew::brew);

pub trait Db:
    salsa::DbWithJar<Jar> + dada_breakpoint::Db + dada_ir::Db + dada_parse::Db + dada_validate::Db
{
}

impl<T> Db for T where
    T: salsa::DbWithJar<Jar>
        + dada_breakpoint::Db
        + dada_ir::Db
        + dada_parse::Db
        + dada_validate::Db
{
}

mod brew;
mod brewery;
mod liveness;
pub mod prelude;
mod scope;

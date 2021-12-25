//! Validates the input syntax tree. Generates various tables.

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

mod validate;

#[salsa::jar(Db)]
pub struct Jar(validate::validate_code);

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

pub mod prelude;

//! Validates the input syntax tree. Generates various tables.

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

#[salsa::jar(db = Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

pub mod breakpoint;
pub mod locations;

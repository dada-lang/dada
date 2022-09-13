//! Validates the input syntax tree. Generates various tables.

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![feature(let_else)]
#![allow(incomplete_features)]

mod validate;

#[salsa::jar(db = Db)]
pub struct Jar(
    validate::root_definitions,
    validate::validate_function,
    validate::validate_function_parameters,
    validate::validate_class_fields,
);

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

pub mod prelude;

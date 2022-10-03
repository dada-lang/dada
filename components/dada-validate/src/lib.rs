//! Validates the input syntax tree. Generates various tables.

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

mod name_lookup;
mod signature;
mod validate;

#[salsa::jar(db = Db)]
pub struct Jar(
    validate::root_definitions,
    validate::validate_function,
    signature::validate_function_signature,
    signature::validate_class_signature,
    signature::validate_class_structure,
);

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db + dada_parse::Db {}

pub mod prelude;

#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

#[salsa::jar(Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db {}

impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db {}

mod data;
mod execute;
mod interpreter;
mod moment;
mod permission;
pub mod prelude;
mod value;

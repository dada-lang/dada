#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

#[macro_use]
mod macro_rules;

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod span;

pub use salsa::Database as Db;

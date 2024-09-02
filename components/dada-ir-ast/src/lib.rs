#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod span;

use salsa::Database as Db;

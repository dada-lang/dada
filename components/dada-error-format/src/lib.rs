#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

mod format;

#[salsa::jar(Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + dada_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + dada_ir::Db {}

pub use format::format_diagnostics;
pub use format::format_diagnostics_with_options;
pub use format::print_diagnostic;
pub use format::FormatOptions;

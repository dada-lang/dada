#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

use std::path::Path;

use ast::Identifier;
use inputs::{CompilationRoot, Krate, SourceFile};
use url::Url;

#[macro_use]
mod macro_rules;

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod span;

#[salsa::db]
pub trait Db: salsa::Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    fn root(&self) -> CompilationRoot;

    /// Load a source-file from the given directory.
    /// The modules is a list of parent modules that translates to a file path.
    fn source_file<'db>(&'db self, krate: Krate, modules: &[Identifier<'db>]) -> SourceFile;

    /// Convert the url into a string suitable for showing the user.
    fn url_display(&self, url: &Url) -> String;

    /// Controls whether type-checking and other parts of the compiler will dump debug logs.
    /// If `None` is returned, no debugging output is emitted.
    /// If `Some` is returned, it should supply a directory where `.json` files will be created.
    /// The `dada_debug` crate will monitor this directory
    /// and serve up the information for use in debugging.
    fn debug_path(&self) -> Option<&Path>;
}

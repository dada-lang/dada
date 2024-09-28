//! "Symbolic IR": High-level, checked representaton. Derived from the AST.
#![feature(trait_upcasting)]

use std::path::Path;

use dada_ir_ast::inputs::{CompilationRoot, SourceFile};
use scope::Scope;

/// Core functionality needed to symbolize.
#[salsa::db]
pub trait Db: salsa::Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    fn root(&self) -> CompilationRoot;

    /// Load a source-file at a given path
    fn source_file(&self, path: &Path) -> Option<SourceFile>;
}

pub mod class;
pub mod expr;
pub mod function;
pub mod indices;
pub mod module;
mod scope;
pub mod symbol;
pub mod ty;

mod symbolize;

pub mod prelude {
    pub trait Symbolize<'db> {
        type Symbolic;

        fn symbolize(self, db: &'db dyn crate::Db) -> Self::Symbolic;
    }
}

trait SymbolizeInScope<'db> {
    type Symbolic;

    fn symbolize_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'db>) -> Self::Symbolic;
}

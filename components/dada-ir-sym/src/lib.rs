//! "Symbolic IR": High-level, checked representaton. Derived from the AST.
#![feature(trait_upcasting)]
// REMOVE THESE:
#![expect(dead_code)]
#![expect(unused_imports)]
#![expect(unused_variables)]

use std::path::Path;

use dada_ir_ast::{
    ast::Identifier,
    inputs::{CompilationRoot, SourceFile},
};
use function::SignatureSymbols;
use scope::Scope;

/// Core functionality needed to symbolize.
#[salsa::db]
pub trait Db: salsa::Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    fn root(&self) -> CompilationRoot;

    /// Load a source-file at a given path
    fn source_file(&self, path: &Path) -> Option<SourceFile>;

    /// Create interned "self" identifier
    fn self_id(&self) -> Identifier<'_>;
}

pub mod class;
pub mod expr;
pub mod function;
pub mod indices;
pub mod module;
mod populate;
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

    fn symbolize_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic;
}

//! "Symbolic IR": High-level, checked representaton. Derived from the AST.
#![feature(trait_upcasting)]

use dada_ir_ast::{
    ast::Identifier,
    inputs::{CompilationRoot, Krate, SourceFile},
};

/// Core functionality needed to symbolize.
#[salsa::db]
pub trait Db: salsa::Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    fn root(&self) -> CompilationRoot;

    /// Load a source-file from the given directory.
    /// The modules is a list of parent modules that translates to a file path.
    fn source_file<'db>(&'db self, krate: Krate, modules: &[Identifier<'db>]) -> SourceFile;
}

pub mod binder;
pub mod class;
pub mod function;
pub mod indices;
pub mod module;
mod populate;
pub mod primitive;
pub mod scope;
pub mod scope_tree;
pub mod subst;
pub mod symbol;
pub mod ty;

pub mod prelude {
    use crate::scope::Scope;

    /// Create the symbol for a given piece of the AST.
    /// This is typically a tracked impl so that invocations are memoized.
    pub trait IntoSymbol<'db> {
        type Symbolic;

        fn into_symbol(self, db: &'db dyn crate::Db) -> Self::Symbolic;
    }

    /// Same as [`IntoSymbol`][] but implemented by enums that are not tracked.
    pub trait ToSymbol<'db> {
        type Symbolic;

        fn to_symbol(&self, db: &'db dyn crate::Db) -> Self::Symbolic;
    }

    impl<'db, T: ToSymbol<'db>> IntoSymbol<'db> for T {
        type Symbolic = T::Symbolic;

        fn into_symbol(self, db: &'db dyn crate::Db) -> Self::Symbolic {
            self.to_symbol(db)
        }
    }

    pub trait IntoSymInScope<'db> {
        type Symbolic;

        fn into_sym_in_scope(
            self,
            db: &'db dyn crate::Db,
            scope: &Scope<'_, 'db>,
        ) -> Self::Symbolic;
    }
}

//! "Symbolic IR": High-level, type checked representaton. Derived from the AST.

#![feature(trait_upcasting)]
#![feature(async_closure)]

use dada_ir_ast::{
    ast::Identifier,
    inputs::{CompilationRoot, Krate, SourceFile},
};
use env::{Env, EnvLike};

/// Core functionality needed to symbolize.
#[salsa::db]
pub trait Db: dada_ir_ast::Db {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    fn root(&self) -> CompilationRoot;

    /// Load a source-file from the given directory.
    /// The modules is a list of parent modules that translates to a file path.
    fn source_file<'db>(&'db self, krate: Krate, modules: &[Identifier<'db>]) -> SourceFile;
}

pub mod binder;
mod blocks;
mod bound;
mod check;
pub mod class;
mod env;
mod exprs;
pub mod function;
pub mod indices;
mod inference;
mod member;
mod misc_tys;
pub mod module;
pub mod object_ir;
mod populate;
pub mod primitive;
pub mod scope;
pub mod scope_tree;
mod signature;
mod statements;
mod subobject;
pub mod subst;
pub mod symbol;
pub mod ty;
mod types;
mod universe;
pub mod well_known;

pub mod prelude {
    use crate::binder::Binder;
    use crate::class::SymField;
    use crate::function::{SymFunction, SymFunctionSignature};
    use crate::object_ir::ObjectExpr;
    use crate::ty::SymTy;
    use dada_ir_ast::diagnostic::Errors;

    /// Return the symbol corresponding to the AST node.
    /// Implementations are memoized so that this can be called many times and will always yield the same symbol.
    pub trait Symbol<'db>: Copy {
        type Output;

        fn symbol(self, db: &'db dyn crate::Db) -> Self::Output;
    }

    pub trait CheckedBody<'db> {
        fn checked_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>>;
    }

    #[salsa::tracked]
    impl<'db> CheckedBody<'db> for SymFunction<'db> {
        #[salsa::tracked]
        fn checked_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>> {
            crate::blocks::check_function_body(db, self)
        }
    }

    pub trait CheckedFieldTy<'db> {
        fn checked_field_ty(self, db: &'db dyn crate::Db) -> Binder<'db, Binder<'db, SymTy<'db>>>;
    }

    #[salsa::tracked]
    impl<'db> CheckedFieldTy<'db> for SymField<'db> {
        #[salsa::tracked]
        fn checked_field_ty(self, db: &'db dyn crate::Db) -> Binder<'db, Binder<'db, SymTy<'db>>> {
            match crate::misc_tys::check_field(db, self) {
                Ok(v) => v,
                Err(reported) => crate::misc_tys::field_err_ty(db, self, reported),
            }
        }
    }
    pub trait CheckedSignature<'db> {
        fn checked_signature(self, db: &'db dyn crate::Db) -> Errors<SymFunctionSignature<'db>>;
    }

    #[salsa::tracked]
    impl<'db> CheckedSignature<'db> for SymFunction<'db> {
        #[salsa::tracked]
        fn checked_signature(self, db: &'db dyn crate::Db) -> Errors<SymFunctionSignature<'db>> {
            crate::signature::check_function_signature(db, self)
        }
    }
}

/// Convert to a type checked representation in the given environment.
/// This is implemented by types that can be converted synchronously
/// (although they may yield an inference variable if parts of the computation
/// had to be deferred).
trait CheckInEnv<'db>: Copy {
    type Output;

    fn check_in_env(self, env: &mut dyn EnvLike<'db>) -> Self::Output;
}

/// Type check an expression (including a block) in the given environment.
/// This is an async operation -- it may block if insufficient inference data is available.
trait CheckExprInEnv<'db> {
    type Output;

    async fn check_expr_in_env(&self, env: &Env<'db>) -> Self::Output;
}

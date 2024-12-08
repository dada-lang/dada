//! "Symbolic IR": High-level, type checked representaton. Derived from the AST.

#![feature(trait_upcasting)]
#![feature(async_closure)]

use dada_ir_ast::{
    ast::Identifier,
    inputs::{CompilationRoot, Krate, SourceFile},
};
use env::{Env, EnvLike};
use scope::Scope;

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
    use crate::IntoSymbol;
    use dada_ir_ast::diagnostic::Errors;

    /// Create the symbol corresponding to some piece of AST IR.
    /// All implementations of this trait are tracked so if you call it multiple times you get the same symbol each time.
    pub trait Symbol<'db> {
        type Symbolic;

        fn symbol(&self, db: &'db dyn crate::Db) -> Self::Symbolic;
    }

    impl<'db, T: IntoSymbol<'db>> Symbol<'db> for T {
        type Symbolic = T::Symbolic;

        fn symbol(&self, db: &'db dyn crate::Db) -> Self::Symbolic {
            self.into_symbol(db)
        }
    }

    pub trait ObjectCheckFunctionBody<'db> {
        fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>>;
    }

    #[salsa::tracked]
    impl<'db> ObjectCheckFunctionBody<'db> for SymFunction<'db> {
        #[salsa::tracked]
        fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>> {
            crate::blocks::check_function_body(db, self)
        }
    }

    pub trait ObjectCheckFieldTy<'db> {
        fn object_check_field_ty(
            self,
            db: &'db dyn crate::Db,
        ) -> Binder<'db, Binder<'db, SymTy<'db>>>;
    }

    pub trait ObjectCheckFunctionSignature<'db> {
        fn object_check_signature(
            self,
            db: &'db dyn crate::Db,
        ) -> Errors<SymFunctionSignature<'db>>;
    }

    #[salsa::tracked]
    impl<'db> ObjectCheckFunctionSignature<'db> for SymFunction<'db> {
        #[salsa::tracked]
        fn object_check_signature(
            self,
            db: &'db dyn crate::Db,
        ) -> Errors<SymFunctionSignature<'db>> {
            crate::signature::check_function_signature(db, self)
        }
    }

    #[salsa::tracked]
    impl<'db> ObjectCheckFieldTy<'db> for SymField<'db> {
        #[salsa::tracked]
        fn object_check_field_ty(
            self,
            db: &'db dyn crate::Db,
        ) -> Binder<'db, Binder<'db, SymTy<'db>>> {
            match crate::misc_tys::check_field(db, self) {
                Ok(v) => v,
                Err(reported) => crate::misc_tys::field_err_ty(db, self, reported),
            }
        }
    }
}

trait Checking<'db> {
    type Checking;

    async fn check(&self, env: &Env<'db>) -> Self::Checking;
}

trait IntoSymInScope<'db> {
    type Symbolic;

    fn into_sym_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic;
}

trait SymbolizeInEnv<'db> {
    type Output;

    fn symbolize_in_env(&self, env: &mut dyn EnvLike<'db>) -> Self::Output;
}
trait IntoSymbol<'db>: Copy {
    type Symbolic;

    fn into_symbol(self, db: &'db dyn crate::Db) -> Self::Symbolic;
}

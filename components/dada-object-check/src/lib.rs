//! This crate is responsible for going from the AST for a function
//! body to the "symbol" version (`SymBlock`). Along the way it performs
//! type checking.

#![feature(trait_upcasting)]
#![feature(async_closure)]

use dada_ir_ast::diagnostic::{Err, Errors};
pub use dada_ir_sym::Db;
use dada_ir_sym::{binder::Binder, class::SymField, function::SymFunction};
use env::Env;
use object_ir::{ObjectExpr, ObjectFunctionSignature, SymTy};

pub mod prelude {
    use dada_ir_ast::diagnostic::Errors;
    use dada_ir_sym::binder::Binder;

    use crate::object_ir::{ObjectExpr, ObjectFunctionSignature, SymTy};

    pub trait ObjectCheckFunctionBody<'db> {
        fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>>;
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
        ) -> Errors<ObjectFunctionSignature<'db>>;
    }
}

mod blocks;
mod bound;
mod check;
mod env;
mod exprs;
mod inference;
mod member;
mod misc_tys;
pub mod object_ir;
mod signature;
mod statements;
mod subobject;
mod types;
mod universe;

trait Checking<'db> {
    type Checking;

    async fn check(&self, env: &Env<'db>) -> Self::Checking;
}

#[salsa::tracked]
impl<'db> prelude::ObjectCheckFunctionBody<'db> for SymFunction<'db> {
    #[salsa::tracked]
    fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>> {
        blocks::check_function_body(db, self)
    }
}

#[salsa::tracked]
impl<'db> prelude::ObjectCheckFieldTy<'db> for SymField<'db> {
    #[salsa::tracked]
    fn object_check_field_ty(self, db: &'db dyn crate::Db) -> Binder<'db, Binder<'db, SymTy<'db>>> {
        match misc_tys::check_field(db, self) {
            Ok(v) => v,
            Err(reported) => self
                .ty(db)
                .map(db, |b| b.map(db, |_ty| SymTy::err(db, reported))),
        }
    }
}

#[salsa::tracked]
impl<'db> prelude::ObjectCheckFunctionSignature<'db> for SymFunction<'db> {
    #[salsa::tracked]
    fn object_check_signature(
        self,
        db: &'db dyn crate::Db,
    ) -> Errors<ObjectFunctionSignature<'db>> {
        blocks::check_function_signature(db, self)
    }
}

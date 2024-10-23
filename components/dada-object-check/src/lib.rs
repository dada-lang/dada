//! This crate is responsible for going from the AST for a function
//! body to the "symbol" version (`SymBlock`). Along the way it performs
//! type checking.

#![feature(trait_upcasting)]
#![feature(async_closure)]
// FIXME
#![expect(dead_code)]
#![expect(unused_variables)]

use check::Check;
use dada_ir_sym::function::SymFunction;
pub use dada_ir_sym::Db;
use env::Env;
use object_ir::ObjectExpr;

pub mod prelude {
    use crate::object_ir::ObjectExpr;

    pub trait ObjectCheckFunctionBody<'db> {
        fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>>;
    }
}

mod blocks;
mod bound;
mod check;
mod env;
mod exprs;
mod inference;
mod member;
pub mod object_ir;
mod statements;
mod subobject;
mod universe;

trait Checking<'db> {
    type Checking;

    async fn check(&self, check: &Check<'db>, env: &Env<'db>) -> Self::Checking;
}

#[salsa::tracked]
impl<'db> prelude::ObjectCheckFunctionBody<'db> for SymFunction<'db> {
    #[salsa::tracked]
    fn object_check_body(self, db: &'db dyn crate::Db) -> Option<ObjectExpr<'db>> {
        blocks::check_function_body(db, self)
    }
}

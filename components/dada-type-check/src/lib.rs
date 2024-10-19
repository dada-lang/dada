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

pub mod prelude {
    use crate::ir::CheckedExpr;

    pub trait CheckFunctionBody<'db> {
        fn check_function_body(self, db: &'db dyn crate::Db) -> Option<CheckedExpr<'db>>;
    }
}

mod blocks;
mod bound;
mod check;
mod env;
mod exprs;
mod inference;
mod ir;
mod member;
mod object_ir;
mod statements;
mod universe;

trait Checking<'db> {
    type Checking;

    async fn check(&self, check: &Check<'db>, env: &Env<'db>) -> Self::Checking;
}

impl<'db> prelude::CheckFunctionBody<'db> for SymFunction<'db> {
    fn check_function_body(self, db: &'db dyn crate::Db) -> Option<ir::CheckedExpr<'db>> {
        blocks::check_function_body(db, self)
    }
}

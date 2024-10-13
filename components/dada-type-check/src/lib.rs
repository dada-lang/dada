//! This crate is responsible for going from the AST for a function
//! body to the "symbol" version (`SymBlock`). Along the way it performs
//! type checking.

#![feature(trait_upcasting)]
// FIXME
#![expect(dead_code)]
#![expect(unused_variables)]

use dada_ir_sym::function::SymFunction;
pub use dada_ir_sym::Db;
use env::Env;
use executor::Check;

pub mod prelude {
    use crate::ir::CheckedExpr;

    pub trait CheckFunctionBody<'db> {
        fn check_function_body(self, db: &'db dyn crate::Db) -> Option<CheckedExpr<'db>>;
    }
}

mod blocks;
mod bound;
mod checking_ir;
mod env;
mod executor;
mod exprs;
mod inference;
mod ir;
mod statements;
mod universe;

trait Checking<'chk, 'db: 'chk> {
    type Checking;

    async fn check(&self, check: &Check<'chk, 'db>, env: &Env<'db>) -> Self::Checking;
}

impl<'db> prelude::CheckFunctionBody<'db> for SymFunction<'db> {
    fn check_function_body(self, db: &'db dyn crate::Db) -> Option<ir::CheckedExpr<'db>> {
        blocks::check_function_body(db, self)
    }
}

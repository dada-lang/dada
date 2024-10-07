//! This crate is responsible for going from the AST for a function
//! body to the "symbol" version (`SymBlock`). Along the way it performs
//! type checking.

use dada_ir_sym::function::SymFunction;
pub use dada_ir_sym::Db;
use env::Env;

pub mod prelude {
    use crate::ir::CheckedBlock;

    pub trait CheckFunctionBody<'db> {
        fn check_function_body(self, db: &'db dyn crate::Db) -> Option<CheckedBlock<'db>>;
    }
}

mod blocks;
mod env;
mod ir;
mod statements;

pub trait CheckInEnv<'db> {
    type Checked;
    fn check_in_env(&self, db: &'db dyn crate::Db, env: &mut Env<'_, 'db>) -> Self::Checked;
}

impl<'db> prelude::CheckFunctionBody<'db> for SymFunction<'db> {
    fn check_function_body(self, db: &'db dyn crate::Db) -> Option<ir::CheckedBlock<'db>> {
        blocks::check_function_body(db, self)
    }
}

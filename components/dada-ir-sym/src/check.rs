//! Defines the type-checking and name-resolution logic. This is what creates the symbolic IR.

use env::{Env, EnvLike};

use crate::ir::types::SymTy;

pub(crate) mod blocks;
mod bound;
mod env;
mod exprs;
pub(crate) mod fields;
mod inference;
mod member_lookup;
mod runtime;
pub(crate) mod scope;
pub(crate) mod scope_tree;
pub(crate) mod signature;
mod statements;
mod subobject;
mod temporaries;
mod types;
mod universe;

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

impl<'db> CheckInEnv<'db> for SymTy<'db> {
    type Output = SymTy<'db>;

    fn check_in_env(self, _env: &mut dyn EnvLike<'db>) -> Self::Output {
        self
    }
}

//! Defines the type-checking and name-resolution logic. This is what creates the symbolic IR.

use env::Env;
use runtime::Runtime;

use crate::ir::types::SymTy;

pub(crate) mod blocks;
mod combinator;
mod env;
mod exprs;
pub(crate) mod fields;
pub(crate) mod functions;
mod inference;
mod member_lookup;
mod modules;
mod places;
mod predicates;
mod report;
mod runtime;
pub(crate) mod scope;
pub(crate) mod scope_tree;
pub(crate) mod signature;
mod statements;
mod subst_impls;
mod subtype;
mod temporaries;
mod to_red;
mod types;
mod universe;

/// Check an expression in a full environment.
/// This is an async operation -- it may block if insufficient inference data is available.
trait CheckInEnv<'db> {
    type Output;

    async fn check_in_env(&self, env: &Env<'db>) -> Self::Output;
}

impl<'db> CheckInEnv<'db> for SymTy<'db> {
    type Output = SymTy<'db>;

    async fn check_in_env(&self, _env: &Env<'db>) -> Self::Output {
        *self
    }
}

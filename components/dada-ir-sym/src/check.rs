//! Defines the type-checking and name-resolution logic. This is what creates the symbolic IR.

use env::Env;
use live_places::LivePlaces;
use runtime::Runtime;

use crate::ir::types::SymTy;

mod alternatives;
pub(crate) mod blocks;
mod debug;
mod env;
mod exprs;
pub(crate) mod fields;
pub(crate) mod functions;
mod inference;
mod live_places;
mod member_lookup;
mod modules;
mod places;
mod predicates;
mod red;
mod report;
mod resolve;
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
trait CheckTyInEnv<'db> {
    type Output;

    async fn check_in_env(&self, env: &mut Env<'db>) -> Self::Output;
}

trait CheckExprInEnv<'db> {
    type Output;

    async fn check_in_env(&self, env: &mut Env<'db>, live_after: LivePlaces) -> Self::Output;
}

impl<'db> CheckTyInEnv<'db> for SymTy<'db> {
    type Output = SymTy<'db>;

    async fn check_in_env(&self, _env: &mut Env<'db>) -> Self::Output {
        *self
    }
}

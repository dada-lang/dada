//! Defines the type-checking and name-resolution logic. This is what creates the symbolic IR.
#![doc = include_str!("../docs/type_checking.md")]

use env::Env;
use live_places::LivePlaces;
use runtime::Runtime;

use crate::ir::types::SymTy;

pub(crate) mod blocks;
mod debug;
mod env;
mod exprs;
pub(crate) mod fields;
pub(crate) mod functions;
mod generics;
pub(crate) mod inference;
mod live_places;
mod member_lookup;
mod modules;
mod places;
pub(crate) mod predicates;
pub(crate) mod red;
pub(crate) mod report;
mod resolve;
mod runtime;
pub(crate) mod scope;
pub(crate) mod scope_tree;
pub(crate) mod signature;
mod statements;
mod stream;
mod subst_impls;
pub(crate) mod subtype;
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

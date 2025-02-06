use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_util::boxed_async_fn;

use crate::ir::types::{SymGenericTerm, SymPerm, SymTy, SymTyKind};

use super::env::Env;

mod require;
mod test;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Predicate {
    Copy,
    Move,
    Owned,
    Lent,
}

impl Predicate {
    pub fn invert(self) -> Predicate {
        match self {
            Predicate::Copy => Predicate::Move,
            Predicate::Move => Predicate::Copy,
            Predicate::Owned => Predicate::Lent,
            Predicate::Lent => Predicate::Owned,
        }
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::Copy => write!(f, "copy"),
            Predicate::Move => write!(f, "move"),
            Predicate::Owned => write!(f, "owned"),
            Predicate::Lent => write!(f, "lent"),
        }
    }
}

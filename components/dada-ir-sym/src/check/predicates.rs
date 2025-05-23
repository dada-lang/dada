//! Permission system and ownership predicates.
#![doc = include_str!("../../docs/permission_system.md")]

pub mod is_provably_lent;
pub mod is_provably_owned;
pub mod is_provably_shared;
pub mod is_provably_unique;
pub mod require_lent;
pub mod require_owned;
pub mod require_shared;
pub mod require_unique;
pub mod require_where_clause;
pub mod var_infer;

use dada_ir_ast::diagnostic::Errors;
use is_provably_lent::term_is_provably_lent;
use is_provably_owned::term_is_provably_owned;
use is_provably_shared::term_is_provably_shared;
use is_provably_unique::term_is_provably_unique;
use require_lent::require_term_is_lent;
use require_owned::require_term_is_owned;
use require_shared::require_term_is_shared;
use require_unique::require_term_is_unique;
use serde::Serialize;

use crate::ir::types::SymGenericTerm;

use super::{env::Env, report::OrElse};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize)]
pub enum Predicate {
    Shared,
    Unique,
    Owned,
    Lent,
}

impl Predicate {
    pub const ALL: [Predicate; 4] = [
        Predicate::Shared,
        Predicate::Unique,
        Predicate::Owned,
        Predicate::Lent,
    ];
    pub const LEN: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        match self {
            Predicate::Shared => 0,
            Predicate::Unique => 1,
            Predicate::Owned => 2,
            Predicate::Lent => 3,
        }
    }

    /// Returns the "opposite" of this predicate. For example, the opposite of
    /// `Copy` is `Move`, and vice versa. It is not possible for `self` and `Self::invert` to both hold
    /// for thr same term.
    pub fn invert(self) -> Option<Predicate> {
        match self {
            Predicate::Shared => Some(Predicate::Unique),
            Predicate::Unique => Some(Predicate::Shared),
            Predicate::Owned => Some(Predicate::Lent),
            Predicate::Lent => Some(Predicate::Owned),
        }
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::Shared => write!(f, "copy"),
            Predicate::Unique => write!(f, "move"),
            Predicate::Owned => write!(f, "owned"),
            Predicate::Lent => write!(f, "lent"),
        }
    }
}

pub(crate) async fn require_term_is<'db>(
    env: &mut Env<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let term: SymGenericTerm<'db> = term.into();
    match predicate {
        Predicate::Shared => require_term_is_shared(env, term, or_else).await,
        Predicate::Unique => require_term_is_unique(env, term, or_else).await,
        Predicate::Owned => require_term_is_owned(env, term, or_else).await,
        Predicate::Lent => require_term_is_lent(env, term, or_else).await,
    }
}

pub(crate) async fn term_is_provably<'db>(
    env: &mut Env<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
) -> Errors<bool> {
    let term: SymGenericTerm<'db> = term.into();
    match predicate {
        Predicate::Shared => term_is_provably_shared(env, term).await,
        Predicate::Unique => term_is_provably_unique(env, term).await,
        Predicate::Owned => term_is_provably_owned(env, term).await,
        Predicate::Lent => term_is_provably_lent(env, term).await,
    }
}

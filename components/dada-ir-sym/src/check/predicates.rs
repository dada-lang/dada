#![expect(dead_code)]

pub mod is_provably_copy;
pub mod is_provably_lent;
pub mod is_provably_move;
pub mod is_provably_owned;
pub mod require_copy;
pub mod require_lent;
pub mod require_move;
pub mod require_owned;
pub mod var_infer;

use dada_ir_ast::diagnostic::Errors;
use is_provably_copy::term_is_provably_copy;
use is_provably_lent::term_is_provably_lent;
use is_provably_move::term_is_provably_move;
use is_provably_owned::term_is_provably_owned;
use require_copy::require_term_is_copy;
use require_lent::require_term_is_lent;
use require_move::require_term_is_move;
use require_owned::require_term_is_owned;
use serde::Serialize;

use crate::ir::types::SymGenericTerm;

use super::{env::Env, inference::Direction, report::OrElse};

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
    pub fn invert(self) -> Predicate {
        match self {
            Predicate::Shared => Predicate::Unique,
            Predicate::Unique => Predicate::Shared,
            Predicate::Owned => Predicate::Lent,
            Predicate::Lent => Predicate::Owned,
        }
    }

    /// The "bound direction" for a predicate indicate how the predicate
    /// interacts with subtyping:
    ///
    /// * [`Shared`](`Predicate::Shared`) and [`Lent`](`Predicate::Lent`) are [`FromBelow`](`Direction::FromBelow`)
    ///   predicates, meaning that if a type `T` is `Copy` or `Lent`, then all *super*types of `T` are also
    ///   `Copy` and `Lent` (these predicates are "viral" in a sense, they can't be upcast away).
    /// * [`Unique`](`Predicate::Unique`) and [`Owned`](`Predicate::Owned`) are [`FromAbove`](`Direction::FromAbove`)
    ///   predicates, meaning that if a type `T` is `Move` or `Owned`, then all *sub*types of `T` are also
    ///   `Move` and `Owned`. This reflects the fact that a `Move` type like `my` can be assigned to a `Copy`
    ///   type like `our`; but you cannot assign an `our` into a `my` slot.
    pub fn bound_direction(self) -> Direction {
        match self {
            Predicate::Shared | Predicate::Lent => Direction::FromBelow,
            Predicate::Unique | Predicate::Owned => Direction::FromAbove,
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

pub(crate) async fn term_is_provably_leased<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.both(
        async |env| term_is_provably_move(env, term).await,
        async |env| term_is_provably_lent(env, term).await,
    )
    .await
}

pub(crate) async fn require_term_is_leased<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_both(
        async |env| require_term_is_move(env, term, or_else).await,
        async |env| require_term_is_lent(env, term, or_else).await,
    )
    .await
}

pub(crate) async fn require_term_is<'db>(
    env: &mut Env<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let term: SymGenericTerm<'db> = term.into();
    match predicate {
        Predicate::Shared => require_term_is_copy(env, term, or_else).await,
        Predicate::Unique => require_term_is_move(env, term, or_else).await,
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
        Predicate::Shared => term_is_provably_copy(env, term).await,
        Predicate::Unique => term_is_provably_move(env, term).await,
        Predicate::Owned => term_is_provably_owned(env, term).await,
        Predicate::Lent => term_is_provably_lent(env, term).await,
    }
}

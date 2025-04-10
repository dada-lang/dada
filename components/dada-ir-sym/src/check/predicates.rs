pub mod is_provably_copy;
pub mod is_provably_lent;
pub mod is_provably_move;
pub mod is_provably_owned;
pub mod isnt_provably_copy;
pub mod require_copy;
pub mod require_isnt_provably_copy;
pub mod require_lent;
pub mod require_move;
pub mod require_owned;
pub mod var_infer;

use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;
use is_provably_copy::{perm_is_provably_copy, red_ty_is_provably_copy};
use is_provably_lent::{perm_is_provably_lent, red_ty_is_provably_lent, term_is_provably_lent};
use is_provably_move::{perm_is_provably_move, red_ty_is_provably_move, term_is_provably_move};
use is_provably_owned::{perm_is_provably_owned, red_ty_is_provably_owned, term_is_provably_owned};
use isnt_provably_copy::perm_isnt_provably_copy;
use require_copy::require_chain_is_copy;
use require_isnt_provably_copy::require_chain_isnt_provably_copy;
use require_lent::{require_chain_is_lent, require_term_is_lent};
use require_move::{require_chain_is_move, require_term_is_move};
use require_owned::{require_chain_is_owned, require_term_is_owned};
use serde::Serialize;
pub use var_infer::{test_perm_infer_is_known_to_be, test_var_is_provably};

use crate::ir::types::SymGenericTerm;

use super::{
    env::Env,
    inference::Direction,
    red::{Lien, RedPerm, RedTy},
    report::OrElse,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize)]
pub enum Predicate {
    Copy,
    Move,
    Owned,
    Lent,
}

impl Predicate {
    pub const ALL: [Predicate; 4] = [
        Predicate::Copy,
        Predicate::Move,
        Predicate::Owned,
        Predicate::Lent,
    ];
    pub const LEN: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        match self {
            Predicate::Copy => 0,
            Predicate::Move => 1,
            Predicate::Owned => 2,
            Predicate::Lent => 3,
        }
    }

    /// Returns the "opposite" of this predicate. For example, the opposite of
    /// `Copy` is `Move`, and vice versa. It is not possible for `self` and `Self::invert` to both hold
    /// for thr same term.
    pub fn invert(self) -> Predicate {
        match self {
            Predicate::Copy => Predicate::Move,
            Predicate::Move => Predicate::Copy,
            Predicate::Owned => Predicate::Lent,
            Predicate::Lent => Predicate::Owned,
        }
    }

    /// The "bound direction" for a predicate indicate how the predicate
    /// interacts with subtyping:
    ///
    /// * [`Copy`](`Predicate::Copy`) and [`Lent`](`Predicate::Lent`) are [`FromBelow`](`Direction::FromBelow`)
    ///   predicates, meaning that if a type `T` is `Copy` or `Lent`, then all *super*types of `T` are also
    ///   `Copy` and `Lent` (these predicates are "viral" in a sense, they can't be upcast away).
    /// * [`Move`](`Predicate::Move`) and [`Owned`](`Predicate::Owned`) are [`FromAbove`](`Direction::FromAbove`)
    ///   predicates, meaning that if a type `T` is `Move` or `Owned`, then all *sub*types of `T` are also
    ///   `Move` and `Owned`. This reflects the fact that a `Move` type like `my` can be assigned to a `Copy`
    ///   type like `our`; but you cannot assign an `our` into a `my` slot.
    pub fn bound_direction(self) -> Direction {
        match self {
            Predicate::Copy | Predicate::Lent => Direction::FromBelow,
            Predicate::Move | Predicate::Owned => Direction::FromAbove,
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

#[boxed_async_fn]
pub async fn require_chain_is<'db>(
    env: &mut Env<'db>,
    chain: &RedPerm<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match predicate {
        Predicate::Copy => require_chain_is_copy(env, chain, or_else).await,
        Predicate::Move => require_chain_is_move(env, chain, or_else).await,
        Predicate::Owned => require_chain_is_owned(env, chain, or_else).await,
        Predicate::Lent => require_chain_is_lent(env, chain, or_else).await,
    }
}

#[boxed_async_fn]
pub async fn require_chain_isnt<'db>(
    env: &mut Env<'db>,
    chain: &RedPerm<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match predicate {
        Predicate::Copy => require_chain_isnt_provably_copy(env, chain, or_else).await,
        Predicate::Move => todo!(),
        Predicate::Owned => todo!(),
        Predicate::Lent => todo!(),
    }
}

#[boxed_async_fn]
pub(crate) async fn red_ty_is_provably<'db>(
    env: &mut Env<'db>,
    red_ty: RedTy<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    match predicate {
        Predicate::Copy => red_ty_is_provably_copy(env, red_ty).await,
        Predicate::Move => red_ty_is_provably_move(env, red_ty).await,
        Predicate::Owned => red_ty_is_provably_owned(env, red_ty).await,
        Predicate::Lent => red_ty_is_provably_lent(env, red_ty).await,
    }
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

pub(crate) async fn require_term_is_my<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_both(
        async |env| require_term_is_move(env, term, or_else).await,
        async |env| require_term_is_owned(env, term, or_else).await,
    )
    .await
}

pub(crate) async fn term_is_provably_my<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.both(
        async |env| term_is_provably_move(env, term).await,
        async |env| term_is_provably_owned(env, term).await,
    )
    .await
}

#[boxed_async_fn]
pub async fn chain_is_provably<'db>(
    env: &mut Env<'db>,
    chain: &RedPerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    let perm = Lien::chain_to_perm(db, chain);
    match predicate {
        Predicate::Copy => perm_is_provably_copy(env, perm).await,
        Predicate::Move => perm_is_provably_move(env, perm).await,
        Predicate::Owned => perm_is_provably_owned(env, perm).await,
        Predicate::Lent => perm_is_provably_lent(env, perm).await,
    }
}

#[boxed_async_fn]
pub async fn chain_isnt_provably<'db>(
    env: &mut Env<'db>,
    chain: &RedPerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    let perm = Lien::chain_to_perm(db, chain);
    match predicate {
        Predicate::Copy => perm_isnt_provably_copy(env, perm).await,
        Predicate::Move => todo!(),
        Predicate::Owned => todo!(),
        Predicate::Lent => todo!(),
    }
}

mod combinator;
pub(crate) mod is_copy;
pub(crate) mod is_lent;
pub(crate) mod is_move;
pub(crate) mod is_owned;
mod report;
pub(crate) mod require_copy;
pub(crate) mod require_lent;
pub(crate) mod require_move;
pub(crate) mod require_owned;
mod var_infer;

use combinator::{both, require_both};
use dada_ir_ast::{diagnostic::Errors, span::Span};
use is_lent::term_is_lent;
use is_move::term_is_move;
use require_lent::require_term_is_lent;
use require_move::require_term_is_move;
pub(crate) use var_infer::{test_infer_is, test_var_is};

use crate::ir::types::SymGenericTerm;

use super::env::Env;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

pub(crate) async fn term_is_leased<'db>(env: &Env<'db>, term: SymGenericTerm<'db>) -> Errors<bool> {
    both(term_is_move(env, term), term_is_lent(env, term)).await
}

pub(crate) async fn require_term_is_leased<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<()> {
    require_both(
        require_term_is_move(env, span, term),
        require_term_is_lent(env, span, term),
    )
    .await
}

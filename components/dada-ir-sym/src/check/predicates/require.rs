use dada_ir_ast::{diagnostic::Errors, span::Span};

use crate::{check::env::Env, ir::types::SymGenericTerm};

use super::Predicate;

mod is_copy;
mod is_lent;
mod is_move;
mod is_owned;

pub(crate) async fn require_term_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    match predicate {
        Predicate::Copy => is_copy::require_term_is_copy(env, span, term).await,
        Predicate::Move => is_move::require_term_is_move(env, span, term).await,
        Predicate::Owned => is_owned::require_term_is_owned(env, span, term).await,
        Predicate::Lent => is_lent::require_term_is_lent(env, span, term).await,
    }
}

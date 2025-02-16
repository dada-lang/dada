use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::env::Env,
    ir::types::{SymGenericTerm, SymPlace},
};

mod is_copy;
mod is_lent;
mod is_move;
mod is_owned;

pub(crate) async fn test_term_is_copy<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    is_copy::term_is_copy(env, term).await
}

pub(crate) async fn test_term_is_lent<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    is_lent::term_is_lent(env, term).await
}

pub(crate) async fn test_term_is_move<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    is_move::term_is_move(env, term).await
}

pub(crate) async fn test_term_is_owned<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    is_owned::term_is_owned(env, term).await
}

pub(crate) async fn test_place_is_move<'db>(env: &Env<'db>, place: SymPlace<'db>) -> Errors<bool> {
    is_move::place_is_move(env, place).await
}

pub(crate) async fn test_place_is_lent<'db>(env: &Env<'db>, place: SymPlace<'db>) -> Errors<bool> {
    is_lent::place_is_lent(env, place).await
}

pub(crate) async fn test_place_is_copy<'db>(env: &Env<'db>, place: SymPlace<'db>) -> Errors<bool> {
    is_copy::place_is_copy(env, place).await
}

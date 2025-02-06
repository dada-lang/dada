use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_util::boxed_async_fn;

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind},
};

#[boxed_async_fn]
pub(crate) async fn test_term_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    todo!()
}

#[boxed_async_fn]
pub(crate) async fn test_ty_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymTy<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    todo!()
}

#[boxed_async_fn]
pub(crate) async fn test_perm_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymPerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    todo!()
}

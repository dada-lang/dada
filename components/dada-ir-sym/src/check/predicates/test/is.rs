use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind},
};

pub(crate) async fn test_term_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    match term {
        SymGenericTerm::Type(sym_ty) => test_ty_is(env, span, sym_ty, predicate).await,
        SymGenericTerm::Perm(sym_perm) => test_perm_is(env, span, sym_perm, predicate).await,
        SymGenericTerm::Place(sym_place) => panic!("test_term_is invoked on place: {sym_place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

#[boxed_async_fn]
pub(crate) async fn test_ty_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    ty: SymTy<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    match (ty.kind(db), predicate) {
        (&SymTyKind::Perm(sym_perm, sym_ty), predicate) => {
            Ok(test_apply_is(env, span, sym_perm.into(), sym_ty.into(), predicate).await?)
        }
        (&SymTyKind::Named(sym_ty_name, ref sym_generic_terms), _) => todo!(),
        (SymTyKind::Infer(infer_var_index), _) => todo!(),
        (&SymTyKind::Var(var), _) => Ok(env.var_is_declared_to_be(var, predicate)),
        (SymTyKind::Never, _) => todo!(),
        (SymTyKind::Error(reported), _) => todo!(),
    }
}

async fn test_apply_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    match predicate {
        Predicate::Copy => Ok(test_term_is(env, span, lhs, predicate).await?
            || test_term_is(env, span, rhs, predicate).await?),
        Predicate::Lent => {
            if test_term_is(env, span, rhs, Predicate::Copy).await? {
                Ok(test_term_is(env, span, rhs, predicate).await?)
            } else {
                Ok(test_term_is(env, span, lhs, predicate).await?
                    || test_term_is(env, span, rhs, predicate).await?)
            }
        }
        Predicate::Owned | Predicate::Move => {
            if test_term_is(env, span, rhs, Predicate::Copy).await? {
                Ok(test_term_is(env, span, rhs, predicate).await?)
            } else {
                Ok(test_term_is(env, span, lhs, predicate).await?
                    && test_term_is(env, span, rhs, predicate).await?)
            }
        }
    }
}

#[boxed_async_fn]
pub(crate) async fn test_perm_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    match (perm.kind(db), predicate) {
        (SymPermKind::Error(reported), _) => Err(reported),

        (SymPermKind::Shared(_), Predicate::Copy | Predicate::Lent) => Ok(true),
        (SymPermKind::Shared(_), Predicate::Move | Predicate::Owned) => Ok(false),

        (SymPermKind::Leased(_), Predicate::Move | Predicate::Lent) => Ok(true),
        (SymPermKind::Leased(_), Predicate::Copy | Predicate::Owned) => Ok(false),

        (&SymPermKind::Apply(lhs, rhs), predicate) => {
            Ok(test_apply_is(env, span, lhs.into(), rhs.into(), predicate).await?)
        }

        (SymPermKind::Var(var), predicate) => Ok(env.var_is_declared_to_be(var, predicate)),

        (SymPermKind::Infer(infer_var_index), predicate) => {}
    }
}

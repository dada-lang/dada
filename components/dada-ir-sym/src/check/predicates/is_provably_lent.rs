use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            var_infer::{test_perm_infer_is_known_to_be, test_var_is_provably},
        },
        red::RedTy,
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTyName},
    },
};

use super::{is_provably_move::place_is_provably_move, var_infer::test_ty_infer_is_known_to_be};

pub async fn term_is_provably_lent<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    let (red_ty, perm) = term.to_red_ty(env);
    env.either(
        async |env| red_ty_is_provably_lent(env, red_ty).await,
        async |env| {
            if let Some(perm) = perm {
                perm_is_provably_lent(env, perm).await
            } else {
                Ok(false)
            }
        },
    )
    .await
}

pub async fn red_ty_is_provably_lent<'db>(env: &mut Env<'db>, ty: RedTy<'db>) -> Errors<bool> {
    let db = env.db();
    match ty {
        RedTy::Infer(infer) => test_ty_infer_is_known_to_be(env, infer, Predicate::Lent).await,
        RedTy::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Lent)),
        RedTy::Never => Ok(false),
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(name, generics) => match name {
            SymTyName::Primitive(_) => Ok(true),
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Struct => {
                    env.exists(generics, async |env, generic| {
                        term_is_provably_lent(env, generic).await
                    })
                    .await
                }
                SymAggregateStyle::Class => Ok(false),
            },
            SymTyName::Future => Ok(false),
            SymTyName::Tuple { arity: _ } => {
                env.exists(generics, async |env, generic| {
                    term_is_provably_lent(env, generic).await
                })
                .await
            }
        },
        RedTy::Perm => Ok(false),
    }
}

async fn application_is_provably_lent<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.either(
        async |env| term_is_provably_lent(env, lhs).await,
        async |env| term_is_provably_lent(env, rhs).await,
    )
    .await
}

#[boxed_async_fn]
pub(crate) async fn perm_is_provably_lent<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),
        SymPermKind::My => Ok(false),
        SymPermKind::Our => Ok(false),
        SymPermKind::Shared(ref places) | SymPermKind::Leased(ref places) => {
            // This one is tricky. If the places are copy,
            // then we will reduce to their chains, but then
            // we would be lent if they are lent; but if they are not
            // copy, we are lent.
            env.either(
                async |env| {
                    env.for_all(places, async |env, &place| {
                        place_is_provably_move(env, place).await
                    })
                    .await
                },
                async |env| {
                    env.exists(places, async |env, &place| {
                        place_is_provably_lent(env, place).await
                    })
                    .await
                },
            )
            .await
        }
        SymPermKind::Apply(lhs, rhs) => {
            Ok(application_is_provably_lent(env, lhs.into(), rhs.into()).await?)
        }
        SymPermKind::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Lent)),
        SymPermKind::Infer(infer) => {
            test_perm_infer_is_known_to_be(env, infer, Predicate::Lent).await
        }
    }
}

pub(crate) async fn place_is_provably_lent<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
) -> Errors<bool> {
    let ty = place.place_ty(env).await;
    term_is_provably_lent(env, ty.into()).await
}

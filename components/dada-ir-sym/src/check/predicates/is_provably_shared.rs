use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{Predicate, var_infer::test_var_is_provably},
        red::RedTy,
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTyName},
    },
};

use super::var_infer::infer_is_provably;

pub async fn term_is_provably_shared<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    let (red_ty, perm) = term.to_red_ty(env);
    let db = env.db();
    match red_ty {
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(name, generics) => match name {
            SymTyName::Primitive(_) => Ok(true),
            SymTyName::Aggregate(aggr) => match aggr.style(db) {
                SymAggregateStyle::Struct => {
                    env.for_all(generics, async |env, generic| {
                        term_is_provably_shared(env, perm.apply_to(db, generic)).await
                    })
                    .await
                }
                SymAggregateStyle::Class => perm_is_provably_shared(env, perm).await,
            },
            SymTyName::Future => perm_is_provably_shared(env, perm).await,
            SymTyName::Tuple { arity: _ } => {
                env.for_all(generics, async |env, generic| {
                    term_is_provably_shared(env, perm.apply_to(db, generic)).await
                })
                .await
            }
        },
        RedTy::Never => Ok(false),
        RedTy::Infer(infer) => infer_is_provably(env, perm, infer, Predicate::Shared).await,
        RedTy::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Shared)),
        RedTy::Perm => perm_is_provably_shared(env, perm).await,
    }
}

async fn application_is_provably_shared<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.either(
        async |env| term_is_provably_shared(env, lhs).await,
        async |env| term_is_provably_shared(env, rhs).await,
    )
    .await
}

#[boxed_async_fn]
pub(crate) async fn perm_is_provably_shared<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),
        SymPermKind::My => Ok(false),
        SymPermKind::Our | SymPermKind::Referenced(_) => Ok(true),
        SymPermKind::Mutable(ref places) => places_are_provably_shared(env, places).await,
        SymPermKind::Apply(lhs, rhs) => {
            Ok(application_is_provably_shared(env, lhs.into(), rhs.into()).await?)
        }
        SymPermKind::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Shared)),
        SymPermKind::Infer(infer) => {
            infer_is_provably(env, SymPerm::my(db), infer, Predicate::Shared).await
        }
        SymPermKind::Or(_, _) => todo!(),
    }
}

#[boxed_async_fn]
async fn places_are_provably_shared<'db>(
    env: &mut Env<'db>,
    places: &[SymPlace<'db>],
) -> Errors<bool> {
    env.for_all(places, async |env, &place| {
        place_is_provably_shared(env, place).await
    })
    .await
}

pub(crate) async fn place_is_provably_shared<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
) -> Errors<bool> {
    let ty = place.place_ty(env).await;
    term_is_provably_shared(env, ty.into()).await
}

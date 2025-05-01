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

use super::var_infer::test_ty_infer_is_known_to_be;

pub async fn term_is_provably_copy<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    let (red_ty, perm) = term.to_red_ty(env);
    env.either(
        async |env| red_ty_is_provably_copy(env, red_ty).await,
        async |env| {
            if let Some(perm) = perm {
                perm_is_provably_copy(env, perm).await
            } else {
                Ok(false)
            }
        },
    )
    .await
}

#[boxed_async_fn]
pub async fn red_ty_is_provably_copy<'db>(env: &mut Env<'db>, ty: RedTy<'db>) -> Errors<bool> {
    let db = env.db();
    match ty {
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(name, generics) => match name {
            SymTyName::Primitive(_) => Ok(true),
            SymTyName::Aggregate(aggr) => match aggr.style(db) {
                SymAggregateStyle::Struct => {
                    env.for_all(generics, async |env, generic| {
                        term_is_provably_copy(env, generic).await
                    })
                    .await
                }
                SymAggregateStyle::Class => Ok(false),
            },
            SymTyName::Future => Ok(false),
            SymTyName::Tuple { arity: _ } => {
                env.for_all(generics, async |env, generic| {
                    term_is_provably_copy(env, generic).await
                })
                .await
            }
        },
        RedTy::Never => Ok(false),
        RedTy::Infer(infer) => test_ty_infer_is_known_to_be(env, infer, Predicate::Copy).await,
        RedTy::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Copy)),
        RedTy::Perm => todo!(),
    }
}

async fn application_is_provably_copy<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.either(
        async |env| term_is_provably_copy(env, lhs).await,
        async |env| term_is_provably_copy(env, rhs).await,
    )
    .await
}

#[boxed_async_fn]
pub(crate) async fn perm_is_provably_copy<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),
        SymPermKind::My => Ok(false),
        SymPermKind::Our | SymPermKind::Shared(_) => Ok(true),
        SymPermKind::Leased(ref places) => places_are_ktb_copy(env, places).await,
        SymPermKind::Apply(lhs, rhs) => {
            Ok(application_is_provably_copy(env, lhs.into(), rhs.into()).await?)
        }
        SymPermKind::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Copy)),
        SymPermKind::Infer(infer) => {
            test_perm_infer_is_known_to_be(env, infer, Predicate::Copy).await
        }
        SymPermKind::Or(_, _) => todo!(),
    }
}

#[boxed_async_fn]
async fn places_are_ktb_copy<'db>(env: &mut Env<'db>, places: &[SymPlace<'db>]) -> Errors<bool> {
    env.for_all(places, async |env, &place| {
        place_is_provably_copy(env, place).await
    })
    .await
}

pub(crate) async fn place_is_provably_copy<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
) -> Errors<bool> {
    let ty = place.place_ty(env).await;
    term_is_provably_copy(env, ty.into()).await
}

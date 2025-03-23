use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            var_infer::{test_infer_is_known_to_be, test_var_is_provably},
        },
        red::RedTy,
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTyName},
    },
};

pub(crate) async fn term_isnt_provably_copy<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    let (red_ty, perm) = term.to_red_ty(env);
    env.both(
        async |env| red_ty_isnt_provably_copy(env, red_ty).await,
        async |env| {
            if let Some(perm) = perm {
                perm_isnt_provably_copy(env, perm).await
            } else {
                Ok(true)
            }
        },
    )
    .await
}

async fn red_ty_isnt_provably_copy<'db>(env: &mut Env<'db>, ty: RedTy<'db>) -> Errors<bool> {
    let db = env.db();
    match ty {
        RedTy::Infer(infer) => Ok(!test_infer_is_known_to_be(env, infer, Predicate::Copy).await),
        RedTy::Var(var) => Ok(!test_var_is_provably(env, var, Predicate::Copy)),
        RedTy::Never => Ok(true),
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(sym_ty_name, generics) => match sym_ty_name {
            SymTyName::Primitive(_) => Ok(false),
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Struct => {
                    env.exists(generics, async |env, generic| {
                        term_isnt_provably_copy(env, generic).await
                    })
                    .await
                }
                SymAggregateStyle::Class => Ok(true),
            },
            SymTyName::Future => Ok(false),
            SymTyName::Tuple { arity: _ } => {
                env.exists(generics, async |env, generic| {
                    term_isnt_provably_copy(env, generic).await
                })
                .await
            }
        },
        RedTy::Perm => Ok(true),
    }
}

async fn application_isnt_provably_copy<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.both(
        async |env| term_isnt_provably_copy(env, lhs).await,
        async |env| term_isnt_provably_copy(env, rhs).await,
    )
    .await
}

#[boxed_async_fn]
pub(crate) async fn perm_isnt_provably_copy<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),
        SymPermKind::My => Ok(true),
        SymPermKind::Our | SymPermKind::Shared(_) => Ok(false),
        SymPermKind::Leased(ref places) => {
            env.exists(places, async |env, &place| {
                place_isnt_provably_copy(env, place).await
            })
            .await
        }

        SymPermKind::Apply(lhs, rhs) => {
            Ok(application_isnt_provably_copy(env, lhs.into(), rhs.into()).await?)
        }

        SymPermKind::Var(var) => Ok(!test_var_is_provably(env, var, Predicate::Copy)),

        SymPermKind::Infer(infer) => {
            Ok(!test_infer_is_known_to_be(env, infer, Predicate::Copy).await)
        }
    }
}

pub(crate) async fn place_isnt_provably_copy<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
) -> Errors<bool> {
    let ty = place.place_ty(env).await;
    term_isnt_provably_copy(env, ty.into()).await
}

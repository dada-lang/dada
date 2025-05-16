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

pub async fn term_is_provably_owned<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    let (red_ty, perm) = term.to_red_ty(env);
    let db = env.db();
    match red_ty {
        RedTy::Infer(infer) => infer_is_provably(env, perm, infer, Predicate::Owned).await,
        RedTy::Var(var) => {
            env.both(
                async |env| Ok(test_var_is_provably(env, var, Predicate::Owned)),
                async |env| perm_is_provably_owned(env, perm).await,
            )
            .await
        }
        RedTy::Never => perm_is_provably_owned(env, perm).await,
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_) => Ok(true),
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Struct => {
                    env.for_all(generics, async |env, &generic| {
                        term_is_provably_owned(env, perm.apply_to(db, generic)).await
                    })
                    .await
                }
                SymAggregateStyle::Class => perm_is_provably_owned(env, perm).await,
            },
            SymTyName::Future => perm_is_provably_owned(env, perm).await,
            SymTyName::Tuple { arity: _ } => {
                env.for_all(generics, async |env, &generic| {
                    term_is_provably_owned(env, perm.apply_to(db, generic)).await
                })
                .await
            }
        },
        RedTy::Perm => perm_is_provably_owned(env, perm).await,
    }
}

async fn application_is_provably_owned<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<bool> {
    env.both(
        async |env| term_is_provably_owned(env, lhs).await,
        async |env| term_is_provably_owned(env, rhs).await,
    )
    .await
}

#[boxed_async_fn]
pub(crate) async fn perm_is_provably_owned<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),
        SymPermKind::My => Ok(true),
        SymPermKind::Our | SymPermKind::Referenced(_) => Ok(false),
        SymPermKind::Mutable(ref places) => {
            env.exists(places, async |env, &place| {
                place_is_provably_owned(env, place).await
            })
            .await
        }

        SymPermKind::Apply(lhs, rhs) => {
            Ok(application_is_provably_owned(env, lhs.into(), rhs.into()).await?)
        }

        SymPermKind::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Owned)),

        SymPermKind::Infer(infer) => {
            infer_is_provably(env, SymPerm::my(db), infer, Predicate::Owned).await
        }

        SymPermKind::Or(_, _) => todo!(),
    }
}

pub(crate) async fn place_is_provably_owned<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
) -> Errors<bool> {
    let ty = place.place_ty(env).await;
    term_is_provably_owned(env, ty.into()).await
}

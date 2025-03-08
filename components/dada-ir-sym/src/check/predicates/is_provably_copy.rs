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
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    },
};

pub(crate) async fn term_is_provably_copy<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<bool> {
    match term {
        SymGenericTerm::Type(sym_ty) => ty_is_provably_copy(env, sym_ty).await,
        SymGenericTerm::Perm(sym_perm) => perm_is_provably_copy(env, sym_perm).await,
        SymGenericTerm::Place(sym_place) => panic!("term_is invoked on place: {sym_place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

#[boxed_async_fn]
async fn ty_is_provably_copy<'db>(env: &mut Env<'db>, ty: SymTy<'db>) -> Errors<bool> {
    let db = env.db();
    match *ty.kind(db) {
        SymTyKind::Perm(sym_perm, sym_ty) => {
            Ok(application_is_provably_copy(env, sym_perm.into(), sym_ty.into()).await?)
        }
        SymTyKind::Infer(infer) => Ok(test_infer_is_known_to_be(env, infer, Predicate::Copy).await),
        SymTyKind::Var(var) => Ok(test_var_is_provably(env, var, Predicate::Copy)),
        SymTyKind::Never => Ok(false),
        SymTyKind::Error(reported) => Err(reported),
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_) => Ok(true),
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Struct => {
                    env.for_all(generics, async |env, &generic| {
                        term_is_provably_copy(env, generic).await
                    })
                    .await
                }
                SymAggregateStyle::Class => Ok(false),
            },
            SymTyName::Future => Ok(false),
            SymTyName::Tuple { arity: _ } => {
                env.for_all(generics, async |env, &generic| {
                    term_is_provably_copy(env, generic).await
                })
                .await
            }
        },
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
            Ok(test_infer_is_known_to_be(env, infer, Predicate::Copy).await)
        }
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
    ty_is_provably_copy(env, ty).await
}

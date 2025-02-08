use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::{
        classes::SymAggregateStyle,
        indices::InferVarIndex,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind, SymTyName},
    },
};

use super::combinator::{Extensions, both, either, exists, for_all, not};

pub(crate) async fn test_term_is<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    term_is(env, term, predicate)
        .and_not(term_is(env, term, predicate.invert()))
        .await
}

async fn term_is<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    match term {
        SymGenericTerm::Type(sym_ty) => ty_is(env, sym_ty, predicate).await,
        SymGenericTerm::Perm(sym_perm) => perm_is(env, sym_perm, predicate).await,
        SymGenericTerm::Place(sym_place) => panic!("term_is invoked on place: {sym_place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

#[boxed_async_fn]
async fn ty_is<'db>(env: &Env<'db>, ty: SymTy<'db>, predicate: Predicate) -> Errors<bool> {
    let db = env.db();
    match *ty.kind(db) {
        SymTyKind::Perm(sym_perm, sym_ty) => {
            Ok(apply_is(env, sym_perm.into(), sym_ty.into(), predicate).await?)
        }
        SymTyKind::Infer(infer) => Ok(infer_is(env, infer, predicate).await),
        SymTyKind::Var(var) => Ok(env.var_is_declared_to_be(var, predicate)),
        SymTyKind::Never => perm_is(env, SymPerm::my(env.db()), predicate).await,
        SymTyKind::Error(reported) => Err(reported),
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_) => match predicate {
                Predicate::Copy | Predicate::Owned => Ok(true),
                Predicate::Move | Predicate::Lent => Ok(false),
            },
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Struct => value_ty_is(env, predicate, generics).await,
                SymAggregateStyle::Class => class_ty_is(env, predicate, generics).await,
            },
            SymTyName::Future => class_ty_is(env, predicate, generics).await,
            SymTyName::Tuple { arity: _ } => value_ty_is(env, predicate, generics).await,
        },
    }
}

async fn value_ty_is<'db>(
    env: &Env<'db>,
    predicate: Predicate,
    generics: &[SymGenericTerm<'db>],
) -> Errors<bool> {
    match predicate {
        Predicate::Move => {
            exists(generics, async |&generic| {
                term_is(env, generic, Predicate::Move).await
            })
            .await
        }
        Predicate::Copy => {
            for_all(generics, async |&generic| {
                term_is(env, generic, Predicate::Copy).await
            })
            .await
        }
        Predicate::Lent => Ok(false),
        Predicate::Owned => {
            for_all(generics, async |&generic| {
                term_is(env, generic, Predicate::Owned).await
            })
            .await
        }
    }
}

async fn class_ty_is<'db>(
    env: &Env<'db>,
    predicate: Predicate,
    generics: &[SymGenericTerm<'db>],
) -> Errors<bool> {
    match predicate {
        Predicate::Move => Ok(true),
        Predicate::Copy => Ok(false),
        Predicate::Lent => Ok(false),
        Predicate::Owned => {
            for_all(generics, async |&generic| {
                term_is(env, generic, Predicate::Owned).await
            })
            .await
        }
    }
}

async fn apply_is<'db>(
    env: &Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    match predicate {
        Predicate::Copy => either(term_is(env, lhs, predicate), term_is(env, rhs, predicate)).await,

        Predicate::Move => {
            both(
                term_is(env, rhs, Predicate::Move),
                term_is(env, lhs, Predicate::Move),
            )
            .await
        }

        Predicate::Lent => {
            either(
                term_is(env, rhs, Predicate::Lent),
                both(
                    not(term_is(env, rhs, Predicate::Copy)),
                    term_is(env, lhs, Predicate::Lent),
                ),
            )
            .await
        }

        Predicate::Owned => {
            both(
                term_is(env, rhs, Predicate::Owned),
                either(
                    term_is(env, rhs, Predicate::Copy),
                    term_is(env, lhs, Predicate::Owned),
                ),
            )
            .await
        }
    }
}

#[boxed_async_fn]
pub(crate) async fn perm_is<'db>(
    env: &Env<'db>,
    perm: SymPerm<'db>,
    predicate: Predicate,
) -> Errors<bool> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),

        // My = Move & Owned
        SymPermKind::My => match predicate {
            Predicate::Move | Predicate::Owned => Ok(true),
            Predicate::Copy | Predicate::Lent => Ok(false),
        },

        // Our = Copy & Owned
        SymPermKind::Our => match predicate {
            Predicate::Copy | Predicate::Owned => Ok(true),
            Predicate::Move | Predicate::Lent => Ok(false),
        },

        // Shared = Copy & Lent
        SymPermKind::Shared(_) => match predicate {
            Predicate::Copy | Predicate::Lent => Ok(true),
            Predicate::Move | Predicate::Owned => Ok(false),
        },

        // Leased = Move & Lent
        SymPermKind::Leased(_) => match predicate {
            Predicate::Move | Predicate::Lent => Ok(true),
            Predicate::Copy | Predicate::Owned => Ok(false),
        },

        SymPermKind::Apply(lhs, rhs) => Ok(apply_is(env, lhs.into(), rhs.into(), predicate).await?),

        SymPermKind::Var(var) => Ok(env.var_is_declared_to_be(var, predicate)),

        SymPermKind::Infer(infer) => Ok(infer_is(env, infer, predicate).await),
    }
}

async fn infer_is<'db>(env: &Env<'db>, infer: InferVarIndex, predicate: Predicate) -> bool {
    let inverted = predicate.invert();

    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let (is, is_inverted) = (data.is(predicate), data.is(inverted));
            if is.is_some() {
                Some(true)
            } else if is_inverted.is_some() {
                Some(false)
            } else {
                None
            }
        })
        .await
}

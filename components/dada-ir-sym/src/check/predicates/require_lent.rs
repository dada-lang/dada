use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        predicates::{
            Predicate,
            var_infer::{require_infer_is, require_var_is},
        },
        report::{Because, OrElse},
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind, SymTyName},
    },
};

use super::{
    is_provably_lent::{place_is_provably_lent, term_is_provably_lent},
    is_provably_move::{place_is_provably_move, term_is_provably_move},
};

pub(crate) async fn require_term_is_lent<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_lent(env, sym_ty, or_else).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_lent(env, sym_perm, or_else).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_application_is_lent<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require(
        async |env| {
            env.either(
                async |env| term_is_provably_lent(env, rhs).await,
                async |env| {
                    env.both(
                        async |env| term_is_provably_move(env, rhs).await,
                        async |env| term_is_provably_lent(env, lhs).await,
                    )
                    .await
                },
            )
            .await
        },
        |env| or_else.report(env, Because::JustSo),
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_lent<'db>(
    env: &mut Env<'db>,
    term: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_application_is_lent(env, sym_perm.into(), sym_ty.into(), or_else).await
        }

        // Never
        SymTyKind::Never => Err(or_else.report(env, Because::NeverIsNotLent)),

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, infer, Predicate::Lent, or_else),
        SymTyKind::Var(var) => require_var_is(env, var, Predicate::Lent, or_else),

        // Named types
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_sym_primitive) => Ok(()),

            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Class => Err(or_else.report(env, Because::JustSo)),
                SymAggregateStyle::Struct => {
                    env.require(
                        async |env| {
                            env.exists(generics, async |env, &generic| {
                                term_is_provably_lent(env, generic).await
                            })
                            .await
                        },
                        |env| or_else.report(env, Because::JustSo),
                    )
                    .await
                }
            },

            SymTyName::Future => Err(or_else.report(env, Because::JustSo)),

            SymTyName::Tuple { arity } => {
                assert_eq!(arity, generics.len());
                env.require(
                    async |env| {
                        env.exists(generics, async |env, &generic| {
                            term_is_provably_lent(env, generic).await
                        })
                        .await
                    },
                    |env| or_else.report(env, Because::JustSo),
                )
                .await
            }
        },
    }
}

#[boxed_async_fn]
async fn require_perm_is_lent<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match *perm.kind(db) {
        // Error cases first
        SymPermKind::Error(reported) => Err(reported),

        // My = Move & Owned
        SymPermKind::My => Err(or_else.report(env, Because::JustSo)),

        // Our = Copy & Owned
        SymPermKind::Our => Err(or_else.report(env, Because::JustSo)),

        // Shared = Copy & Lent, Leased = Move & Lent
        SymPermKind::Shared(ref places) | SymPermKind::Leased(ref places) => {
            // This one is tricky. If the places are copy,
            // then we will reduce to their chains, but then
            // we would be lent if they are lent; but if they are not
            // copy, we are lent.
            env.require(
                async |env| {
                    env.for_all(places, async |env, &place| {
                        env.either(
                            // If the place `p` is move, then the result will be `shared[p]` or `leased[p]` perm,
                            // which is lent.
                            async |env| place_is_provably_move(env, place).await,
                            // Or, if the place `p` is not move and hence may be copy, then it must itself be `lent`.
                            async |env| place_is_provably_lent(env, place).await,
                        )
                        .await
                    })
                    .await
                },
                |env| or_else.report(env, Because::JustSo),
            )
            .await
        }

        // Apply
        SymPermKind::Apply(lhs, rhs) => {
            require_application_is_lent(env, lhs.into(), rhs.into(), or_else).await
        }

        // Variable and inference
        SymPermKind::Var(var) => require_var_is(env, var, Predicate::Lent, or_else),
        SymPermKind::Infer(infer) => require_infer_is(env, infer, Predicate::Lent, or_else),
        SymPermKind::Or(_, _) => todo!(),
    }
}

use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        predicates::{
            Predicate,
            var_infer::{require_infer_is, require_var_is},
        },
        red::RedTy,
        report::{Because, OrElse},
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy},
    },
};

use super::{
    is_provably_lent::{place_is_provably_lent, term_is_provably_lent},
    is_provably_unique::{place_is_provably_unique, term_is_provably_unique},
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
                        async |env| term_is_provably_unique(env, rhs).await,
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
    env.indent("require_ty_is_lent", &[&term], async |env| {
        let db = env.db();
        let (red_ty, perm) = term.to_red_ty(env);
        match red_ty {
            // Error cases first
            RedTy::Error(reported) => Err(reported),

            // Never
            RedTy::Never => require_perm_is_lent(env, perm, or_else).await,

            // Inference
            RedTy::Infer(infer) => {
                require_infer_is(env, perm, infer, Predicate::Lent, or_else).await
            }

            // Generic variable
            RedTy::Var(var) => {
                if env.var_is_declared_to_be(var, Predicate::Lent) {
                    Ok(())
                } else if env.var_is_declared_to_be(var, Predicate::Unique) {
                    // If the perm is not known to be unique,
                    // it might be a shared type, in which case,
                    // even if `perm` is lent it doesn't matter.
                    require_perm_is_lent(env, perm, or_else).await
                } else {
                    Err(or_else.report(env, Because::NoWhereClause(var, Predicate::Lent)))
                }
            }

            // Named types
            RedTy::Named(sym_ty_name, _) => match sym_ty_name.style(db) {
                SymAggregateStyle::Struct => {
                    // Structs are never *lent*.
                    //
                    // They can always have at least some content
                    // that is owned.
                    Err(or_else.report(env, Because::StructsAreNotLent(sym_ty_name)))
                }
                SymAggregateStyle::Class => {
                    // For a class to be lent, the permission must be lent.
                    //
                    // We don't consider `Vec[ref[x] String]` to be lent
                    // (though it is also not owned).
                    require_perm_is_lent(env, perm, or_else).await
                }
            },

            RedTy::Perm => require_perm_is_lent(env, perm, or_else).await,
        }
    })
    .await
}

#[boxed_async_fn]
async fn require_perm_is_lent<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_perm_is_lent", &[&perm], async |env| {
        let db = env.db();
        match *perm.kind(db) {
            // Error cases first
            SymPermKind::Error(reported) => Err(reported),

            // My = Move & Owned
            SymPermKind::My => Err(or_else.report(env, Because::JustSo)),

            // Our = Copy & Owned
            SymPermKind::Our => Err(or_else.report(env, Because::JustSo)),

            // Shared = Copy & Lent, Mutable = Move & Lent
            SymPermKind::Referenced(ref places) | SymPermKind::Mutable(ref places) => {
                // This one is tricky. If the places are copy,
                // then we will reduce to their chains, but then
                // we would be lent if they are lent; but if they are not
                // copy, we are lent.
                env.require(
                    async |env| {
                        env.for_all(places, async |env, &place| {
                            env.either(
                                // If the place `p` is move, then the result will be `shared[p]` or `mutable[p]` perm,
                                // which is lent.
                                async |env| place_is_provably_unique(env, place).await,
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
            SymPermKind::Infer(infer) => {
                require_infer_is(env, SymPerm::my(db), infer, Predicate::Lent, or_else).await
            }
            SymPermKind::Or(_, _) => todo!(),
        }
    })
    .await
}

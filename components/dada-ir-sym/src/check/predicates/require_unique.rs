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

use super::is_provably_unique::{place_is_provably_unique, term_is_provably_unique};

pub(crate) async fn require_term_is_unique<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_unique(env, sym_ty, or_else).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_unique(env, sym_perm, or_else).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

#[boxed_async_fn]
async fn require_ty_is_unique<'db>(
    env: &mut Env<'db>,
    term: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_ty_is_unique", &[&term], async |env| {
        let db = env.db();
        let (red_ty, perm) = term.to_red_ty(env);
        match red_ty {
            // Error cases first
            RedTy::Error(reported) => Err(reported),

            // Never
            RedTy::Never => require_perm_is_unique(env, perm, or_else).await,

            // Variable and inference
            RedTy::Infer(infer) => {
                require_infer_is(env, perm, infer, Predicate::Unique, or_else).await
            }

            RedTy::Var(var) => {
                // The variable must be unique, but in that case, the permission must also be
                env.require_both(
                    async |env| require_perm_is_unique(env, perm, or_else).await,
                    async |env| require_var_is(env, var, Predicate::Unique, or_else),
                )
                .await
            }

            // Named types
            RedTy::Named(sym_ty_name, ref generics) => match sym_ty_name.style(db) {
                SymAggregateStyle::Struct => {
                    require_some_generic_is_unique(env, perm, generics, or_else).await
                }
                SymAggregateStyle::Class => require_perm_is_unique(env, perm, or_else).await,
            },

            RedTy::Perm => require_perm_is_unique(env, perm, or_else).await,
        }
    })
    .await
}

async fn require_some_generic_is_unique<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    generics: &[SymGenericTerm<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    env.require(
        async |env| {
            env.exists(generics, async |env, &generic| {
                term_is_provably_unique(env, perm.apply_to(db, generic)).await
            })
            .await
        },
        |env| or_else.report(env, Because::JustSo),
    )
    .await
}

#[boxed_async_fn]
async fn require_perm_is_unique<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_perm_is_unique", &[&perm], async |env| {
        let db = env.db();
        match *perm.kind(db) {
            SymPermKind::Error(reported) => Err(reported),

            SymPermKind::My => Ok(()),

            SymPermKind::Our => Err(or_else.report(env, Because::JustSo)),

            SymPermKind::Referenced(_) => Err(or_else.report(env, Because::JustSo)),

            SymPermKind::Mutable(ref places) => {
                // If there is at least one place `p` that is move, this will result in a `mutable[p]` chain.
                env.require(
                    async |env| {
                        env.exists(places, async |env, &place| {
                            place_is_provably_unique(env, place).await
                        })
                        .await
                    },
                    |env| or_else.report(env, Because::LeasedFromCopyIsCopy(places.to_vec())),
                )
                .await
            }

            // Apply
            SymPermKind::Apply(lhs, rhs) => {
                env.require_both(
                    async |env| require_perm_is_unique(env, lhs, or_else).await,
                    async |env| require_perm_is_unique(env, rhs, or_else).await,
                )
                .await
            }

            // Variable and inference
            SymPermKind::Var(var) => require_var_is(env, var, Predicate::Unique, or_else),
            SymPermKind::Infer(infer) => {
                require_infer_is(env, perm, infer, Predicate::Unique, or_else).await
            }

            SymPermKind::Or(_, _) => todo!(),
        }
    })
    .await
}

use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            var_infer::{require_infer_is, require_var_is},
        },
        red::RedTy,
        report::OrElse,
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy},
    },
};

use super::{is_provably_shared::term_is_provably_shared, require_shared::require_place_is_shared};

pub(crate) async fn require_term_is_owned<'db>(
    env: &mut Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_owned(env, sym_ty, or_else).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_owned(env, sym_perm, or_else).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_both_are_owned<'db>(
    env: &mut Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_both(
        async |env| require_term_is_owned(env, rhs, or_else).await,
        async |env| {
            // this isn't *perfect* -- if we can prove that the `lhs` is owned, we don't
            // need to be able to conclude whether `rhs` is copy or not.
            //
            // not sure if I have the right combinator for this =)
            if !term_is_provably_shared(env, rhs).await? {
                require_term_is_owned(env, lhs, or_else).await
            } else {
                Ok(())
            }
        },
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_owned<'db>(
    env: &mut Env<'db>,
    term: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_ty_is_owned", &[&term], async |env| {
        let db = env.db();
        let (red_ty, perm) = term.to_red_ty(env);
        match red_ty {
            // Error cases first
            RedTy::Error(reported) => Err(reported),

            // Never
            RedTy::Never => require_perm_is_owned(env, perm, or_else).await,

            // Inference
            RedTy::Infer(infer) => {
                require_infer_is(env, perm, infer, Predicate::Owned, or_else).await
            }

            // Generic variables
            RedTy::Var(var) => {
                env.require_both(
                    async |env| require_perm_is_owned(env, perm, or_else).await,
                    async |env| require_var_is(env, var, Predicate::Owned, or_else),
                )
                .await
            }

            // Named types: owned if all their generics are owned
            RedTy::Named(sym_ty_name, ref generics) => match sym_ty_name.style(db) {
                SymAggregateStyle::Struct => {
                    require_generics_are_owned(env, perm, generics, or_else).await
                }
                SymAggregateStyle::Class => {
                    env.require_both(
                        async |env| require_perm_is_owned(env, perm, or_else).await,
                        async |env| {
                            require_generics_are_owned(env, SymPerm::my(db), generics, or_else)
                                .await
                        },
                    )
                    .await
                }
            },

            RedTy::Perm => require_perm_is_owned(env, perm, or_else).await,
        }
    })
    .await
}

async fn require_generics_are_owned<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    generics: &[SymGenericTerm<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    env.require_for_all(generics, async |env, &generic| {
        require_term_is_owned(env, perm.apply_to(db, generic), or_else).await
    })
    .await
}

#[boxed_async_fn]
async fn require_perm_is_owned<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_perm_is_owned", &[&perm], async |env| {
        let db = env.db();
        match *perm.kind(db) {
            // Error cases first
            SymPermKind::Error(reported) => Err(reported),

            // My = Move & Owned
            SymPermKind::My => Ok(()),

            // Our = Copy & Owned
            SymPermKind::Our => Ok(()),

            // Shared = Copy & Lent, Mutable = Move & Lent
            SymPermKind::Referenced(ref places) | SymPermKind::Mutable(ref places) => {
                // In order for a shared[p] or mutable[p] type to be owned,
                // the `p` values must be `our` -- copy so that the shared/mutable
                // doesn't apply, and then themselves owned.
                env.require_for_all(places, async |env, &place| {
                    env.require_both(
                        async |env| require_place_is_shared(env, place, or_else).await,
                        async |env| require_place_is_owned(env, place, or_else).await,
                    )
                    .await
                })
                .await
            }

            // Apply
            SymPermKind::Apply(lhs, rhs) => {
                require_both_are_owned(env, lhs.into(), rhs.into(), or_else).await
            }

            // Variable and inference
            SymPermKind::Var(var) => require_var_is(env, var, Predicate::Owned, or_else),
            SymPermKind::Infer(infer) => {
                require_infer_is(env, SymPerm::my(db), infer, Predicate::Owned, or_else).await
            }

            SymPermKind::Or(_, _) => todo!(),
        }
    })
    .await
}

async fn require_place_is_owned<'db>(
    env: &mut Env<'db>,
    place: SymPlace<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent("require_place_is_owned", &[&place], async |env| {
        let ty = place.place_ty(env).await;
        require_ty_is_owned(env, ty, or_else).await
    })
    .await
}

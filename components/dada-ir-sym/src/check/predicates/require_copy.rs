use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors},
    span::Span,
};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        combinator::{require_both, require_for_all},
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            report::{report_never_must_be_but_isnt, report_term_must_be_but_isnt},
            var_infer::{require_infer_is, require_var_is},
        },
        report::Because,
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    },
};

use super::is_provably_copy::term_is_provably_copy;

pub(crate) async fn require_term_is_copy<'db>(
    env: &Env<'db>,
    term: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_copy(env, sym_ty, or_else).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_copy(env, sym_perm, or_else).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_either_is_copy<'db>(
    env: &Env<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // Simultaneously test for whether LHS/RHS is `predicate`.
    // If either is, we are done.
    // If either is *not*, the other must be.
    require_both(
        async {
            if !term_is_provably_copy(env, rhs).await? {
                require_term_is_copy(env, lhs, or_else).await?;
            }
            Ok(())
        },
        async {
            if !term_is_provably_copy(env, lhs).await? {
                require_term_is_copy(env, rhs, or_else).await?;
            }
            Ok(())
        },
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_copy<'db>(
    env: &Env<'db>,
    term: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_either_is_copy(env, sym_perm.into(), sym_ty.into(), or_else).await
        }

        // Never
        SymTyKind::Never => Err(or_else(Because::NeverIsNotCopy).report(env.db())),

        // Inference variables
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Copy),

        // Universal variables
        SymTyKind::Var(var) => require_var_is(env, span, var, Predicate::Copy),

        // Named types
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_sym_primitive) => Ok(()),

            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Class => {
                    Err(or_else(Because::ClassIsNotCopy(sym_ty_name)).report(env.db()))
                }
                SymAggregateStyle::Struct => {
                    require_for_all(generics, async |&generic| {
                        require_term_is_copy(env, generic, &|because| {
                            or_else(because.struct_component_not_copy(sym_ty_name, generic))
                        })
                        .await
                    })
                    .await
                }
            },

            SymTyName::Future => {
                Err(or_else(Because::ClassIsNotCopy(sym_ty_name)).report(env.db()))
            }

            SymTyName::Tuple { arity } => {
                assert_eq!(arity, generics.len());
                require_for_all(generics, async |&generic| {
                    require_term_is_copy(env, generic, &|because| {
                        or_else(because.struct_component_not_copy(sym_ty_name, generic))
                    })
                    .await
                })
                .await
            }
        },
    }
}

#[boxed_async_fn]
async fn require_perm_is_copy<'db>(
    env: &Env<'db>,
    perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match *perm.kind(db) {
        SymPermKind::Error(reported) => Err(reported),

        SymPermKind::My => Err(report_term_must_be_but_isnt(
            env,
            span,
            perm,
            Predicate::Copy,
        )),

        SymPermKind::Our => Ok(()),

        SymPermKind::Shared(_) => Ok(()),

        SymPermKind::Leased(ref places) => {
            // For a leased[p] to be copy, all the places in `p` must have copy permission.
            require_for_all(places, async |&place| {
                require_place_is_copy(env, span, place).await
            })
            .await
        }

        // Apply
        SymPermKind::Apply(lhs, rhs) => {
            require_either_is_copy(env, span, lhs.into(), rhs.into()).await
        }

        // Variable and inference
        SymPermKind::Var(var) => require_var_is(env, span, var, Predicate::Copy),
        SymPermKind::Infer(infer) => require_infer_is(env, span, infer, or_else),
    }
}

pub(crate) async fn require_place_is_copy<'db>(
    env: &Env<'db>,
    place: SymPlace<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let ty = place.place_ty(env).await;
    require_ty_is_copy(env, ty, or_else).await
}

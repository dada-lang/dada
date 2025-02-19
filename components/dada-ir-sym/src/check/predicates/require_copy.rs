use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            combinator::{require_both, require_for_all},
            report::{report_never_must_be_but_isnt, report_term_must_be_but_isnt},
            var_infer::{require_infer_is, require_var_is},
        },
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    },
};

use super::is_copy::term_is_copy;

pub(crate) async fn require_term_is_copy<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_copy(env, span, sym_ty).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_copy(env, span, sym_perm).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_either_is_copy<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<()> {
    // Simultaneously test for whether LHS/RHS is `predicate`.
    // If either is, we are done.
    // If either is *not*, the other must be.
    require_both(
        async {
            if !term_is_copy(env, rhs).await? {
                require_term_is_copy(env, span, lhs).await?;
            }
            Ok(())
        },
        async {
            if !term_is_copy(env, lhs).await? {
                require_term_is_copy(env, span, rhs).await?;
            }
            Ok(())
        },
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_copy<'db>(env: &Env<'db>, span: Span<'db>, term: SymTy<'db>) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_either_is_copy(env, span, sym_perm.into(), sym_ty.into()).await
        }

        // Never
        SymTyKind::Never => Err(report_never_must_be_but_isnt(env, span, Predicate::Copy)),

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Copy),
        SymTyKind::Var(var) => require_var_is(env, span, var, Predicate::Copy),

        // Named types
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_sym_primitive) => Ok(()),

            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Class => Err(report_term_must_be_but_isnt(
                    env,
                    span,
                    term,
                    Predicate::Copy,
                )),
                SymAggregateStyle::Struct => {
                    require_for_all(generics, async |&generic| {
                        require_term_is_copy(env, span, generic).await
                    })
                    .await
                }
            },

            SymTyName::Future => Err(report_term_must_be_but_isnt(
                env,
                span,
                term,
                Predicate::Copy,
            )),

            SymTyName::Tuple { arity } => {
                assert_eq!(arity, generics.len());
                require_for_all(generics, async |&generic| {
                    require_term_is_copy(env, span, generic).await
                })
                .await
            }
        },
    }
}

#[boxed_async_fn]
async fn require_perm_is_copy<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
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
        SymPermKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Copy),
    }
}

pub(crate) async fn require_place_is_copy<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    place: SymPlace<'db>,
) -> Errors<()> {
    let ty = place.place_ty(env).await;
    require_ty_is_copy(env, span, ty).await
}

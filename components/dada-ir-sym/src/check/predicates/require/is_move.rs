use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            combinator::{do_both, exists, require, require_for_all},
            report::{report_never_must_be_but_isnt, report_term_must_be_but_isnt},
            test::test_term_is_move,
            var_infer::{require_infer_is, require_var_is},
        },
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    },
};

pub(crate) async fn require_term_is_move<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_move(env, span, sym_ty).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_move(env, span, sym_perm).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` is `move`.
/// This requires both `lhs` and `rhs` to be `move` independently.
async fn require_application_is_move<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<()> {
    // Simultaneously test for whether LHS/RHS is `predicate`.
    // If either is, we are done.
    // If either is *not*, the other must be.
    do_both(
        require_term_is_move(env, span, lhs),
        require_term_is_move(env, span, rhs),
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_move<'db>(env: &Env<'db>, span: Span<'db>, term: SymTy<'db>) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_application_is_move(env, span, sym_perm.into(), sym_ty.into()).await
        }

        // Never
        SymTyKind::Never => Err(report_never_must_be_but_isnt(env, span, Predicate::Move)),

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Move),
        SymTyKind::Var(var) => require_var_is(env, span, var, Predicate::Move),

        // Named types
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_sym_primitive) => Err(report_term_must_be_but_isnt(
                env,
                span,
                term,
                Predicate::Move,
            )),

            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Class => Ok(()),
                SymAggregateStyle::Struct => {
                    require(
                        exists(generics, async |&generic| {
                            test_term_is_move(env, generic).await
                        }),
                        || report_term_must_be_but_isnt(env, span, term, Predicate::Move),
                    )
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
                require(
                    exists(generics, async |&generic| {
                        test_term_is_move(env, generic).await
                    }),
                    || report_term_must_be_but_isnt(env, span, term, Predicate::Move),
                )
                .await
            }
        },
    }
}

#[boxed_async_fn]
async fn require_perm_is_move<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
) -> Errors<()> {
    let db = env.db();
    match *perm.kind(db) {
        // Error cases first
        SymPermKind::Error(reported) => Err(reported),

        // My = Move & Owned
        SymPermKind::My => Err(report_term_must_be_but_isnt(
            env,
            span,
            perm,
            Predicate::Copy,
        )),

        // Our = Copy & Owned
        SymPermKind::Our => Ok(()),

        // Shared = Copy & Lent
        SymPermKind::Shared(_) => Ok(()),

        // Leased = Move & Lent
        SymPermKind::Leased(ref places) => {
            require_for_all(places, async |&place| {
                require_place_is_move(env, span, place).await
            })
            .await
        }

        // Apply
        SymPermKind::Apply(lhs, rhs) => {
            require_application_is_move(env, span, lhs.into(), rhs.into()).await
        }

        // Variable and inference
        SymPermKind::Var(var) => require_var_is(env, span, var, Predicate::Move),
        SymPermKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Move),
    }
}

pub(super) async fn require_place_is_move<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    place: SymPlace<'db>,
) -> Errors<()> {
    let ty = place.place_ty(env).await;
    require_ty_is_move(env, span, ty).await
}

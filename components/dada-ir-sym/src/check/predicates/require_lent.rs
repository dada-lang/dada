use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            combinator::{both, either, exists, for_all, require, require_for_all},
            report::{report_never_must_be_but_isnt, report_term_must_be_but_isnt},
            var_infer::{require_infer_is, require_var_is},
        },
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    },
};

use super::{
    is_lent::{place_is_lent, term_is_lent},
    is_move::{place_is_move, term_is_move},
};

pub(crate) async fn require_term_is_lent<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_lent(env, span, sym_ty).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_lent(env, span, sym_perm).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_application_is_lent<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<()> {
    require(
        either(
            term_is_lent(env, rhs),
            both(term_is_move(env, rhs), term_is_lent(env, lhs)),
        ),
        || report_term_must_be_but_isnt(env, span, term, Predicate::Lent),
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_lent<'db>(env: &Env<'db>, span: Span<'db>, term: SymTy<'db>) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_application_is_lent(env, span, term.into(), sym_perm.into(), sym_ty.into())
                .await
        }

        // Never
        SymTyKind::Never => Err(report_never_must_be_but_isnt(env, span, Predicate::Lent)),

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Lent),
        SymTyKind::Var(var) => require_var_is(env, span, var, Predicate::Lent),

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
                        require_term_is_lent(env, span, generic).await
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
                require(
                    exists(generics, async |&generic| term_is_lent(env, generic).await),
                    || report_term_must_be_but_isnt(env, span, term, Predicate::Lent),
                )
                .await
            }
        },
    }
}

#[boxed_async_fn]
async fn require_perm_is_lent<'db>(
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
        SymPermKind::Our => Err(report_term_must_be_but_isnt(
            env,
            span,
            perm,
            Predicate::Lent,
        )),

        // Shared = Copy & Lent, Leased = Move & Lent
        SymPermKind::Shared(ref places) | SymPermKind::Leased(ref places) => {
            // This one is tricky. If the places are copy,
            // then we will reduce to their chains, but then
            // we would be lent if they are lent; but if they are not
            // copy, we are lent.
            require(
                for_all(places, async |&place| {
                    either(
                        // If the place `p` is move, then the result will be `shared[p]` or `leased[p]` perm,
                        // which is lent.
                        place_is_move(env, place),
                        // Or, if the place `p` is not move and hence may be copy, then it must itself be `lent`.
                        place_is_lent(env, place),
                    )
                    .await
                }),
                || report_term_must_be_but_isnt(env, span, perm, Predicate::Lent),
            )
            .await
        }

        // Apply
        SymPermKind::Apply(lhs, rhs) => {
            require_application_is_lent(env, span, perm.into(), lhs.into(), rhs.into()).await
        }

        // Variable and inference
        SymPermKind::Var(var) => require_var_is(env, span, var, Predicate::Lent),
        SymPermKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Lent),
    }
}

pub(super) async fn require_place_is_lent<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    place: SymPlace<'db>,
) -> Errors<()> {
    let ty = place.place_ty(env).await;
    require_ty_is_lent(env, span, ty).await
}

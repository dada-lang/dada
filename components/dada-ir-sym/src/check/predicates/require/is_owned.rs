use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        places::PlaceTy,
        predicates::{
            Predicate,
            combinator::{do_both, require_for_all},
            report::report_term_must_be_but_isnt,
            require::is_copy::require_place_is_copy,
            var_infer::{require_infer_is, require_var_is},
        },
    },
    ir::types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind},
};

pub(crate) async fn require_term_is_owned<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is_owned(env, span, sym_ty).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is_owned(env, span, sym_perm).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_both_are_owned<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
) -> Errors<()> {
    // Simultaneously test for whether LHS/RHS is `predicate`.
    // If either is, we are done.
    // If either is *not*, the other must be.
    do_both(
        require_term_is_owned(env, span, lhs),
        require_term_is_owned(env, span, rhs),
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is_owned<'db>(env: &Env<'db>, span: Span<'db>, term: SymTy<'db>) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(lhs, rhs) => {
            require_both_are_owned(env, span, lhs.into(), rhs.into()).await
        }

        // Never
        SymTyKind::Never => Ok(()),

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Owned),
        SymTyKind::Var(var) => require_var_is(env, span, var, Predicate::Owned),

        // Named types: owned if all their generics are owned
        SymTyKind::Named(_sym_ty_name, ref generics) => {
            require_for_all(generics, async |&generic| {
                require_term_is_owned(env, span, generic).await
            })
            .await
        }
    }
}

#[boxed_async_fn]
async fn require_perm_is_owned<'db>(
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

        // Shared = Copy & Lent, Leased = Move & Lent
        SymPermKind::Shared(ref places) | SymPermKind::Leased(ref places) => {
            // In order for a shared[p] or leased[p] type to be owned,
            // the `p` values must be `our` -- copy so that the shared/leased
            // doesn't apply, and then themselves owned.
            require_for_all(places, async |&place| {
                do_both(
                    require_place_is_copy(env, span, place),
                    require_place_is_owned(env, span, place),
                )
                .await
            })
            .await
        }

        // Apply
        SymPermKind::Apply(lhs, rhs) => {
            require_both_are_owned(env, span, lhs.into(), rhs.into()).await
        }

        // Variable and inference
        SymPermKind::Var(var) => require_var_is(env, span, var, Predicate::Owned),
        SymPermKind::Infer(infer) => require_infer_is(env, span, infer, Predicate::Owned),
    }
}

async fn require_place_is_owned<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    place: SymPlace<'db>,
) -> Errors<()> {
    let ty = place.place_ty(env).await;
    require_ty_is_owned(env, span, ty).await
}

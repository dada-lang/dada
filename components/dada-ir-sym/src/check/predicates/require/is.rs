use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        predicates::{Predicate, test::is::test_term_is},
    },
    ir::{
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind},
        variables::SymVariable,
    },
};

pub(crate) async fn require_term_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is(env, span, sym_ty, predicate).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is(env, span, sym_perm, predicate).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_apply_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    match predicate {
        Predicate::Move | Predicate::Owned => {
            env.defer_for_all(span, [lhs, rhs], async move |env, term| {
                require_term_is(env, span, term, predicate).await
            });
            Ok(())
        }
        Predicate::Copy => {
            if !test_term_is(env, span, rhs, Predicate::Copy).await? {
                require_term_is(env, span, lhs, Predicate::Copy).await?;
            }
            Ok(())
        }
        Predicate::Lent => {
            if test_term_is(env, span, rhs, Predicate::Copy).await? {
                // If RHS is copy, then (LHS RHS) is equivalent to (RHS)
                // and therefore the RHS must be `lent` but `lhs` doesn't matter.
                require_term_is(env, span, rhs, Predicate::Lent).await?;
            } else {
                // Otherwise, (LHS RHS) is equivalent to (LHS \cup RHS)
                // and therefore at least one must be `lent`.
                require_either_is(env, span, lhs, rhs, Predicate::Lent).await?;
            }
            Ok(())
        }
    }
}

/// Requires that either `lhs` or `rhs` satisfies the given predicate.
async fn require_either_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    env.defer(span, async move |env| -> Errors<()> {
        if !test_term_is(env, span, rhs, predicate).await? {
            require_term_is(env, span, lhs, predicate).await?;
        }
        Ok(())
    });
    env.defer(span, async move |env| -> Errors<()> {
        if !test_term_is(env, span, lhs, predicate).await? {
            require_term_is(env, span, rhs, predicate).await?;
        }
        Ok(())
    });
    Ok(())
}

#[boxed_async_fn]
async fn require_ty_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymTy<'db>,
    predicate: Predicate,
) -> Errors<()> {
    let db = env.db();
    match (*term.kind(db), predicate) {
        // If we've already reported an error, ignore
        (SymTyKind::Error(reported), _) => Err(reported),

        // The never type is considered `my`
        (SymTyKind::Never, Predicate::Move | Predicate::Owned) => Ok(()),
        (SymTyKind::Never, Predicate::Copy | Predicate::Lent) => {
            Err(report_never_must_be_but_isnt(env, span, predicate))
        }

        (SymTyKind::Perm(sym_perm, sym_ty), predicate) => {
            require_apply_is(env, span, sym_perm.into(), sym_ty.into(), predicate).await
        }

        (SymTyKind::Named(sym_ty_name, ref generics), Predicate::Owned) => {
            // To be OWNED a type must have no mention of any places etc.

            env.defer_for_all(span, generics.iter().copied(), async |env, generic| {
                require_term_is(env, span, generic.into(), predicate).await
            });
            Ok(())
        }

        (SymTyKind::Named(sym_ty_name, ref generics), Predicate::Copy) => {
            todo!()
        }

        (SymTyKind::Named(sym_ty_name, ref generics), Predicate::Lent) => {
            // Classes are lent iff their permission is lent.
            // Structs are lent iff a field is lent.
            todo!()
        }

        (SymTyKind::Named(sym_ty_name, ref generics), Predicate::Move) => {
            // Classes are move iff their permission is move.
            // Structs are move iff a field is move.
            todo!()
        }

        (SymTyKind::Infer(infer), predicate) => {
            env.require_infer_to_be(span, infer, predicate).await
        }

        (SymTyKind::Var(var), predicate) => require_var_is(env, span, var, predicate),
    }
}

#[boxed_async_fn]
async fn require_perm_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    let db = env.db();
    match (*perm.kind(db), predicate) {
        (SymPermKind::Error(reported), _) => Err(reported),

        (SymPermKind::My, Predicate::Move | Predicate::Owned) => Ok(()),

        (SymPermKind::My, Predicate::Copy | Predicate::Lent) => {
            Err(report_perm_must_be_but_isnt(env, span, perm, predicate))
        }

        (SymPermKind::Our, Predicate::Copy | Predicate::Owned) => Ok(()),

        (SymPermKind::Our, Predicate::Move | Predicate::Lent) => {
            Err(report_perm_must_be_but_isnt(env, span, perm, predicate))
        }

        (SymPermKind::Apply(lhs, rhs), predicate) => {
            require_apply_is(env, span, lhs.into(), rhs.into(), predicate).await
        }

        (SymPermKind::Shared(_), Predicate::Copy | Predicate::Lent)
        | (SymPermKind::Leased(_), Predicate::Move | Predicate::Lent) => Ok(()),

        (SymPermKind::Shared(_), Predicate::Move | Predicate::Owned)
        | (SymPermKind::Leased(_), Predicate::Copy | Predicate::Owned) => {
            Err(report_perm_must_be_but_isnt(env, span, perm, predicate))
        }

        (SymPermKind::Var(var), predicate) => require_var_is(env, span, var, predicate),

        (SymPermKind::Infer(infer), predicate) => {
            env.require_infer_to_be(span, infer, predicate).await
        }
    }
}

fn require_var_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Errors<()> {
    if env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(report_var_must_be_but_is_not_declared_to_be(
            env, span, var, predicate,
        ))
    }
}

fn report_never_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the never type (`!`) is not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("the never type (`!`) is considered `my` and therefore is not `{predicate}`"),
    )
    .report(db)
}

fn report_perm_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the permission `{perm}` cannot be `{predicate}`"),
    )
    .report(db)
}

fn report_var_must_be_but_is_not_declared_to_be<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the generic variable `{var}` is not declared to be `{predicate}`"),
    )
    .report(db)
}

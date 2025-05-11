use dada_ir_ast::diagnostic::{Diagnostic, Errors, Level, Reported};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::{Predicate, term_is_provably},
        report::{Because, OrElse},
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

use super::require_term_is;

pub fn test_var_is_provably<'db>(
    env: &mut Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> bool {
    env.var_is_declared_to_be(var, predicate)
}

pub(super) fn require_var_is<'db>(
    env: &mut Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(or_else.report(env, Because::VarNotDeclaredToBe(var, predicate)))
    }
}

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub async fn require_infer_is<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if env.infer_is(infer, predicate).is_some() {
        // Already required to meet this predicate.
        return Ok(());
    }

    if let Some(_or_else_invert) = env.infer_is(infer, predicate.invert()) {
        // Already required NOT to meet this predicate.
        return Err(or_else.report(env, Because::JustSo)); // FIXME we can do better than JustSo
    }

    // Record that `infer` is required to meet the predicate.
    env.set_infer_is(infer, predicate, or_else);

    // Enforce the result implications of that
    match predicate {
        Predicate::Lent => {
            // If `infer` must be lent, its upper bounds must be lent.
            // Lower bounds don't have to be.
            //
            // FIXME: Well, lower bounds have to be "maybe" lent.
            require_bounding_terms_are(
                env,
                infer,
                Some(Direction::FromAbove),
                Predicate::Lent,
                or_else,
            )
            .await
        }

        Predicate::Owned => {
            // If `infer` must be owned, its lower bounds must be owned.
            // Upper bounds don't have to be.
            //
            // FIXME: Well, upper bounds have to be "maybe" owned.
            require_bounding_terms_are(
                env,
                infer,
                Some(Direction::FromBelow),
                Predicate::Owned,
                or_else,
            )
            .await
        }

        Predicate::Shared | Predicate::Move => {
            require_bounding_terms_are(env, infer, None, predicate, or_else).await
        }
    }
}

async fn require_bounding_terms_are<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    direction: Option<Direction>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let mut bounds = env.term_bounds(infer, direction);
    while let Some((_, bound)) = bounds.next(env).await {
        require_term_is(env, bound, predicate, or_else).await?;
    }
    Ok(())
}

/// Wait until we know whether the inference variable IS the given predicate
/// or we know that we'll never be able to prove that it is.
#[boxed_async_fn]
pub async fn infer_is_provably<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);

    match predicate {
        Predicate::Shared | Predicate::Move => {
            // Copy/move predicates are preserved by up/downcasting:
            //
            // * All copy perms (`our`, `ref[_]`, `our mut[_]`, `copy perm X`)
            //   are only subtypes of other copy perms.
            // * All move perms (`my`, `mut[_]`, `move perm X`)
            //   are only subtypes of other move perms.
            //
            // Therefore, if any lower or upper bound meets the predicate, then,
            // we know the predicate must hold.
            //
            // Similarly, if any lower or upper bound is known NOT to meet the predicate,
            // then the predicate cannot hold: e.g.,:
            //
            // * if we have `perm X` as a lower or upper bound
            //   and nothing is known about `X`, then we will never be able to say that
            //   this variable is `copy` (or `owned`, etc).
            // * if we have a lower bound of `our`, then we know the variable
            //   will never be `move`.

            let mut bounds = env.term_bounds(infer, None);
            if let Some((_, bound)) = bounds.next(env).await {
                term_is_provably(env, bound, predicate).await
            } else {
                // We never got any inference bound, so we can't say anything useful.
                Ok(false)
            }
        }

        Predicate::Lent => {
            // "Lent" predicates are influenced by the fact that `our` (owned) is a subtype of `ref[x]` and
            // other lent predicates.
            let mut bounds = env.term_bounds(infer, None);
            while let Some((direction, bound)) = bounds.next(env).await {
                match direction {
                    Direction::FromBelow
                        if term_is_provably(env, bound, Predicate::Lent).await? =>
                    {
                        return Ok(true);
                    }

                    Direction::FromAbove
                        if term_is_provably(env, bound, Predicate::Owned).await? =>
                    {
                        return Ok(false);
                    }

                    _ => {
                        // Not clear yet whether result is true or false.
                        //
                        // FIXME: We could "fail faster" than this in some cases.
                        // For example, given a lower bound of `perm X` (not `lent perm X`)
                        // we can be sure that it will never be tightened to anything `lent`,
                        // but for now we wait until the end to return `false`.
                    }
                }
            }
            Ok(false)
        }

        Predicate::Owned => {
            // An "owned" perm can be upcast into a "lent" perm.
            // e.g., `our <: ref[x]` for any `x`.
            //
            // So an *owned* lower bound (e.g., `our`)
            // does not imply the result is owned,
            // as a second lower bound that is lent (e.g., `ref[x]`) could
            // come later, and the lub of `our` and `ref[x]` is `ref[x]`.
            //
            // Similarly, a non-owned upper bound (e.g., `ref[x]`)
            // does not imply the result is NOT owned,
            // as a second upper bound that IS owned (e.g., `our) could
            // come later, and the glb of `our` and `ref[x]` is `our`.
            //
            // However, a *non-owned* lower bound (e.g., `ref[X]` or `perm X`)
            // DOES imply the result cannot be owned. Even if an owned lower bound
            // comes later, the lub will still not be owned.
            //
            // Conversely, an *owned* upper bound (e.g., `our`)
            // implies the result MUST be owned.  Even if a lent upper bound
            // comes later, the glb will still be owned.
            let mut bounds = env.term_bounds(infer, None);
            while let Some((direction, bound)) = bounds.next(env).await {
                match direction {
                    Direction::FromBelow
                        if term_is_provably(env, bound, Predicate::Lent).await? =>
                    {
                        return Ok(false);
                    }

                    Direction::FromAbove
                        if term_is_provably(env, bound, Predicate::Owned).await? =>
                    {
                        return Ok(true);
                    }

                    _ => {
                        // Not clear yet whether result is true or false.
                        //
                        // FIXME: We could "fail faster" than this in some cases.
                        // For example, given a lower bound of `perm X` (not `owned perm X`)
                        // we can be sure that it will never be tightened to anything `owned`,
                        // but for now we wait until the end to return `false`.
                    }
                }
            }
            Ok(false)
        }
    }
}

fn report_type_annotations_needed<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    let span = env.infer_var_span(infer);
    env.report(
    Diagnostic::error(db, span, "type annotation needed")
        .label(
            db,
            Level::Error,
            span,
            "I could not infer the correct type here, can you annotate it?",
        )
        .child(Diagnostic::info(
            db,
            span,
            format!(
                "I was trying to figure out whether the type was `{predicate}` and I got stuck :("
            ),
        ))
    )
}

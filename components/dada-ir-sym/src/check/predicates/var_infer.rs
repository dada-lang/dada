use dada_ir_ast::diagnostic::{Diagnostic, Errors, Level, Reported};

use crate::{
    check::{
        debug::TaskDescription,
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::Predicate,
        report::{ArcOrElse, Because, OrElse},
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

use super::red_ty_is_provably;

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
pub fn require_infer_is<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // FIXME
}

/// Wait until we know that the inference variable IS (or IS NOT) the given predicate.
pub async fn test_ty_infer_is_known_to_be(
    env: &mut Env<'_>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);

    // XXX
}

/// Wait until we know whether the inference variable IS the given predicate
/// or we know that we'll never be able to prove that it is.
pub async fn test_perm_infer_is_known_to_be<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);

    match predicate {
        Predicate::Copy | Predicate::Move => {
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

            if let Some((_, bound)) = env.next_perm_bound(infer, None, &mut None).await {
                bound.is_provably(env, predicate)
            } else {
                // We never got any inference bound, so we can't say anything useful.
                Ok(false)
            }
        }

        Predicate::Lent => {
            // "Lent" predicates are influenced by the fact that `our` (owned) is a subtype of `ref[x]` and
            // other lent predicates.
            Ok(env
                .find_red_perm_bound(infer, None, async |env, direction, bound| match direction {
                    Direction::FromBelow => {
                        if bound.is_provably(env, Predicate::Lent)? {
                            // `ref[x] <: ?X` or `mut[x] <: ?X` or `lent perm Y <: ?X`:
                            // This implies that `?X` must be lent.
                            Ok(Some(true))
                        } else if bound.is_our(env)? {
                            // `our <: ?X` could later be upcast to `ref[x]` or `our mut[x]`
                            Ok(None)
                        } else {
                            // `my <: ?X` or `perm Y <: ?X` -- `?X` cannot become something known to be lent
                            Ok(Some(false))
                        }
                    }

                    Direction::FromAbove => {
                        // In an upper bound...
                        if bound.is_provably(env, Predicate::Lent)? {
                            if bound.is_provably(env, Predicate::Copy)? {
                                // `?X <: ref[x]`-- `?X` could be `our`, so this this does
                                // not tell us anything useful.
                                Ok(None)
                            } else {
                                // `?X <: mut[x]` -- `?X` must be `mut[x]`, so must be lent
                                Ok(Some(true))
                            }
                        } else {
                            // `?X <: our` or `?X <: X`
                            //
                            // `?X` cannot be `ref[x]` and friends.
                            Ok(Some(false))
                        }
                    }
                })
                .await?
                .unwrap_or(false))
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
            Ok(env
                .find_red_perm_bound(infer, None, async |env, direction, bound| match direction {
                    Direction::FromAbove => {
                        // `?X <: B`...
                        if bound.is_provably(env, Predicate::Owned)? {
                            Ok(Some(true))
                        } else if bound.is_provably(env, Predicate::Copy)? {
                            // A non-owned copy bound could be `ref[x]`, `our mut[x]`,
                            // or `copy perm P`, and in any of those cases,
                            // it could be tightened to `our`.
                            Ok(None)
                        } else {
                            // If not owned nor copy (or not *known* to be copy), then
                            // the bound must be either `mut[x]` or `perm P`,
                            // and in either case, it can't be tightened to something owned.
                            Ok(Some(false))
                        }
                    }

                    Direction::FromBelow => {
                        // `B <: ?X`
                        if bound.is_provably(env, Predicate::Owned)? {
                            if bound.is_provably(env, Predicate::Copy)? {
                                // `our <: ?X` does not imply `?X` is owned.
                                // It could be tightened to `ref[x]`.
                                Ok(None)
                            } else {
                                // `my <: ?X` or `owned perm P <: ?X` both
                                // imply `?X` must be owned.
                                Ok(Some(true))
                            }
                        } else {
                            // If `ref[x] <: ?X` or `mut[x] <: ?X` etc,
                            // then `?X` cannot be owned.
                            Ok(Some(false))
                        }
                    }
                })
                .await?
                .unwrap_or(false))
        }
    }
}

async fn require_bounds_provably_predicate<'db>(
    env: &mut Env<'db>,
    perm_infer: InferVarIndex,
    direction: Direction,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_for_all_red_perm_bounds(perm_infer, Some(direction), async |env, _, red_perm| {
        if red_perm.is_provably(env, predicate)? {
            Ok(())
        } else {
            Err(or_else.report(env, Because::JustSo))
        }
    })
    .await
}

fn report_type_annotations_needed<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    let span = env.infer_var_span(infer);
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
        .report(db)
}

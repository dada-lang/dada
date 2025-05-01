use dada_ir_ast::diagnostic::{Diagnostic, Errors, Level, Reported};

use crate::{
    check::{
        debug::TaskDescription,
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::Predicate,
        red::RedPerm,
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

pub(super) fn require_var_isnt<'db>(
    env: &mut Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if !env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(or_else.report(env, Because::VarDeclaredToBe(var, predicate)))
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
    let (is_already, isnt_already) = env.runtime().with_inference_var_data(infer, |data| {
        (
            data.is_known_to_provably_be(predicate),
            data.is_known_not_to_provably_be(predicate),
        )
    });

    // Check if we are already required to be the predicate.
    if is_already.is_some() {
        return Ok(());
    }

    // Check if were already required to not be the predicate
    // and report an error if so.
    if let Some(prev_or_else) = isnt_already {
        return Err(or_else.report(env, Because::InferredIsnt(predicate, prev_or_else)));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    if let Some(or_else) = env.require_inference_var_is(infer, predicate, or_else) {
        defer_require_bounds_provably_predicate(env, infer, predicate, or_else);

        let (is_move, is_copy, is_owned) = env.runtime().with_inference_var_data(infer, |data| {
            (
                data.is_known_to_provably_be(Predicate::Move).is_some(),
                data.is_known_to_provably_be(Predicate::Copy).is_some(),
                data.is_known_to_provably_be(Predicate::Owned).is_some(),
            )
        });

        if let Predicate::Move | Predicate::Owned = predicate
            && is_move
            && is_owned
        {
            // If we just learned that the inference variable must be `my`...
        }

        if let Predicate::Copy | Predicate::Owned = predicate
            && is_copy
            && is_owned
        {
            // If we just learned that the inference variable must be `our`...
        }
    }

    Ok(())
}

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub(super) fn require_infer_isnt<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let (is_already, isnt_already) = env.runtime().with_inference_var_data(infer, |data| {
        (
            data.is_known_to_provably_be(predicate),
            data.is_known_not_to_provably_be(predicate),
        )
    });

    // Check if we are already required not to be the predicate.
    if isnt_already.is_some() {
        return Ok(());
    }

    // Check if were already required to be the predicate
    // and report an error if so.
    if let Some(prev_or_else) = is_already {
        return Err(or_else.report(env, Because::InferredIs(predicate, prev_or_else)));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    if let Some(or_else) = env.require_inference_var_isnt(infer, predicate, or_else) {
        defer_require_bounds_not_provably_predicate(env, infer, predicate, or_else);
    }

    Ok(())
}

/// Wait until we know that the inference variable IS (or IS NOT) the given predicate.
pub async fn test_ty_infer_is_known_to_be(
    env: &mut Env<'_>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);
    let mut storage = None;
    loop {
        let Some((is, isnt, bound)) = env
            .watch_inference_var(
                infer,
                |data| {
                    (
                        data.is_known_to_provably_be(predicate).is_some(),
                        data.is_known_not_to_provably_be(predicate).is_some(),
                        data.red_ty_bound(predicate.bound_direction())
                            .map(|pair| pair.0),
                    )
                },
                &mut storage,
            )
            .await
        else {
            return Err(report_type_annotations_needed(env, infer, predicate));
        };

        if is {
            return Ok(true);
        } else if isnt {
            return Ok(false);
        } else if let Some(bound) = bound {
            return red_ty_is_provably(env, bound, predicate).await;
        }
    }
}

/// Wait until we know whether the inference variable IS the given predicate
/// or we know that we'll never be able to prove that it is.
pub async fn test_perm_infer_is_known_to_be<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);
    let mut storage = None;
    loop {
        let Some((is, isnt, lower_red_perms, upper_red_perms)) = env
            .watch_inference_var(
                infer,
                |data| {
                    (
                        data.is_known_to_provably_be(predicate).is_some(),
                        data.is_known_not_to_provably_be(predicate).is_some(),
                        data.red_perm_bounds(Direction::FromBelow)
                            .iter()
                            .map(|pair| pair.0.clone())
                            .collect::<Vec<RedPerm<'db>>>(),
                        data.red_perm_bounds(Direction::FromAbove)
                            .iter()
                            .map(|pair| pair.0.clone())
                            .collect::<Vec<RedPerm<'db>>>(),
                    )
                },
                &mut storage,
            )
            .await
        else {
            return Err(report_type_annotations_needed(env, infer, predicate));
        };

        if is {
            return Ok(true);
        } else if isnt {
            return Ok(false);
        } else {
            match predicate {
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

                    if lower_red_perms
                        .iter()
                        .any(|perm| !perm.is_provably(env, Predicate::Owned))
                    {
                        return Ok(false);
                    }

                    if upper_red_perms
                        .iter()
                        .any(|perm| perm.is_provably(env, Predicate::Owned))
                    {
                        return Ok(true);
                    }
                }

                Predicate::Copy | Predicate::Move | Predicate::Lent => {
                    // Other predicates cannot be lost through upcasting:
                    //
                    // * All copy perms (`our`, `ref[_]`, `our mut[_]`, `copy perm X`)
                    //   are only subtypes of other copy perms.
                    // * All move perms (`my`, `mut[_]`, `move perm X`)
                    //   are only subtypes of other copy perms.
                    // * All lent perms (`ref[_]`, `mut[_]`, `lent perm X`)
                    //   are only subtypes of other copy perms.
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

                    let bounding_perms = || lower_red_perms.iter().chain(upper_red_perms.iter());

                    if bounding_perms().any(|perm| !perm.is_provably(env, predicate)) {
                        return Ok(false);
                    }

                    if bounding_perms().any(|perm| perm.is_provably(env, predicate)) {
                        return Ok(true);
                    }
                }
            }
        }
    }
}

fn defer_require_bounds_provably_predicate<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: ArcOrElse<'db>,
) {
    let perm_infer = env.perm_infer(infer);
    env.spawn(
        TaskDescription::RequireBoundsProvablyPredicate(infer, predicate),
        async move |env| match predicate {
            Predicate::Owned => {
                // For Owned, we require lower bounds to be owned.
                // If you have a lower bound of `ref[x]`, the result
                // cannot be `our`.
                //
                // But we cannot require upper bounds to be owned.
                // You could have a upper bound of `ref[x]` and the result could
                // still be inferred to `our` since `our <: ref[x]`.
                require_bounds_provably_predicate(
                    env,
                    perm_infer,
                    Direction::FromBelow,
                    predicate,
                    &or_else,
                )
                .await
            }
            Predicate::Copy | Predicate::Move | Predicate::Lent => {
                env.require_both(
                    async |env| {
                        require_bounds_provably_predicate(
                            env,
                            perm_infer,
                            Direction::FromAbove,
                            predicate,
                            &or_else,
                        )
                        .await
                    },
                    async |env| {
                        require_bounds_provably_predicate(
                            env,
                            perm_infer,
                            Direction::FromBelow,
                            predicate,
                            &or_else,
                        )
                        .await
                    },
                )
                .await
            }
        },
    );
}

fn defer_require_bounds_not_provably_predicate<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: ArcOrElse<'db>,
) {
    let perm_infer = env.perm_infer(infer);
    env.spawn(
        TaskDescription::RequireBoundsNotProvablyPredicate(infer, predicate),
        async move |env| {
            // As above, if we want to prove that something *isn't* `Copy`,
            // we need to ensure that the supertype isn't `Copy`.
            //
            // To show that it isn't `Move`, either suffices.
            env.require_for_all_red_perm_bounds(infer, Direction::FromAbove, async |env, chain| {
                match predicate {
                    Predicate::Owned => {
                        // If something must not be provably Owned,
                        // then it cannot have an Owned upper bound.
                        // Owned lower bounds like `our` are ok as it could still be inferred to `ref[_]`.
                        require_bounds_not_provably_predicate(
                            env,
                            perm_infer,
                            Direction::FromAbove,
                            predicate,
                            &or_else,
                        )
                        .await
                    }
                    Predicate::Copy | Predicate::Move | Predicate::Lent => {
                        env.require_both(
                            async |env| {
                                require_bounds_provably_predicate(
                                    env,
                                    perm_infer,
                                    Direction::FromAbove,
                                    predicate,
                                    &or_else,
                                )
                                .await
                            },
                            async |env| {
                                require_bounds_provably_predicate(
                                    env,
                                    perm_infer,
                                    Direction::FromBelow,
                                    predicate,
                                    &or_else,
                                )
                                .await
                            },
                        )
                        .await
                    }
                }
            })
            .await
        },
    );
}

async fn require_bounds_provably_predicate<'db>(
    env: &mut Env<'db>,
    perm_infer: InferVarIndex,
    direction: Direction,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_for_all_red_perm_bounds(perm_infer, direction, async |env, red_perm| {
        if red_perm.is_provably(env, predicate) {
            Ok(())
        } else {
            Err(or_else.report(env, Because::JustSo))
        }
    })
    .await
}

async fn require_bounds_not_provably_predicate<'db>(
    env: &mut Env<'db>,
    perm_infer: InferVarIndex,
    direction: Direction,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.require_for_all_red_perm_bounds(perm_infer, direction, async |env, red_perm| {
        if !red_perm.is_provably(env, predicate) {
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

use dada_ir_ast::diagnostic::{Diagnostic, Errors, Level, Reported};

use crate::{
    check::{
        debug::TaskDescription,
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::{Predicate, chain_is_provably, chain_isnt_provably},
        red::RedPerm,
        report::{ArcOrElse, Because, OrElse},
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

use super::{red_ty_is_provably, require_chain_is, require_chain_isnt};

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

/// Wait until we know that the inference variable IS (or IS NOT) the given predicate.
pub async fn test_perm_infer_is_known_to_be<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);
    let bound_direction = predicate.bound_direction();
    let mut storage = None;
    loop {
        let Some((is, isnt, lower_chains, upper_chains)) = env
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
            // I'm not 100% sure about this part.
            match bound_direction {
                // `FromBelow` bounds are bound like `Copy` or `Lent`, which cannot be lost
                // through upcasting. Therefore, if we find any lower chain for which the predicate
                // holds, then the final type must meet this predicate. Similarly, if we can find
                // an upper chain bound for which the predicate does NOT hold, then there is an error.
                Direction::FromBelow => {
                    if env
                        .exists(lower_chains, async |env, chain| {
                            chain_is_provably(env, &chain, predicate).await
                        })
                        .await?
                    {
                        return Ok(true);
                    }

                    if env
                        .exists(upper_chains, async |env, chain| {
                            chain_isnt_provably(env, &chain, predicate).await
                        })
                        .await?
                    {
                        return Ok(true);
                    }
                }

                Direction::FromAbove => {
                    if env
                        .exists(lower_chains, async |env, chain| {
                            chain_isnt_provably(env, &chain, predicate).await
                        })
                        .await?
                    {
                        return Ok(false);
                    }
                }
            };
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
        async move |env| {
            env.require_for_all_red_perm_bounds(
                perm_infer,
                // We need to ensure that the *supertype* bound meets the predicate.
                // This doesn't really depend on the predicate.
                //
                // Consider the cases:
                //
                // * `Copy` -- it's ok to have subtype bounds that are move as long as the
                //   final type is upcast into a copy value.
                // * `move` -- if supertype is move, subtype must also be move.
                //
                // Analogous reasoning applies to `lent` and `owned`.
                Direction::FromAbove,
                async |env, chain| require_chain_is(env, &chain, predicate, &or_else).await,
            )
            .await
        },
    );
}

fn defer_require_bounds_not_provably_predicate<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: ArcOrElse<'db>,
) {
    env.spawn(
        TaskDescription::RequireBoundsNotProvablyPredicate(infer, predicate),
        async move |env| {
            // As above, if we want to prove that something *isn't* `Copy`,
            // we need to ensure that the supertype isn't `Copy`.
            //
            // To show that it isn't `Move`, either suffices.
            env.require_for_all_red_perm_bounds(infer, Direction::FromAbove, async |env, chain| {
                require_chain_isnt(env, &chain, predicate, &or_else).await
            })
            .await
        },
    );
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

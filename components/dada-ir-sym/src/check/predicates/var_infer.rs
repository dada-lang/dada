use std::sync::Arc;

use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{
        debug::TaskDescription,
        env::Env,
        inference::InferenceVarData,
        predicates::Predicate,
        report::{Because, OrElse},
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

use super::{
    require_copy::require_chain_is_copy,
    require_isnt_provably_copy::require_chain_isnt_provably_copy,
    require_lent::require_chain_is_lent, require_move::require_chain_is_move,
    require_owned::require_chain_is_owned,
};

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
pub async fn test_infer_is_known_to_be(
    env: &mut Env<'_>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> bool {
    env.loop_on_inference_var(infer, |data| {
        let (is, isnt) = (
            data.is_known_to_provably_be(predicate),
            data.is_known_not_to_provably_be(predicate),
        );
        if is.is_some() {
            Some(true)
        } else if isnt.is_some() {
            Some(false)
        } else {
            // We do not yet have a constraint on whether the inference variable
            // is known to be `predicate`, so block to see what new constraints
            // are added in the future.
            None
        }
    })
    .await
    .unwrap_or({
        // If `None` is returned, it indicates that we terminated without ever
        // adding a constrain on the inference variable one way or the other.
        // This implies that the variable is not KNOWN to be `predicate` (though of course
        // it may be).
        false
    })
}

fn defer_require_bounds_provably_predicate<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: Arc<dyn OrElse<'db> + 'db>,
) {
    let perm_infer = env.perm_infer(infer);
    env.spawn(
        TaskDescription::RequireBoundsProvablyPredicate(infer, predicate),
        async move |env| match predicate {
            Predicate::Copy => {
                env.require_for_all_infer_bounds(
                    perm_infer,
                    InferenceVarData::upper_chains,
                    async |env, chain| require_chain_is_copy(env, &chain, &or_else).await,
                )
                .await
            }
            Predicate::Move => {
                env.require_for_all_infer_bounds(
                    perm_infer,
                    InferenceVarData::lower_chains,
                    async |env, chain| require_chain_is_move(env, &chain, &or_else).await,
                )
                .await
            }
            Predicate::Owned => {
                env.require_for_all_infer_bounds(
                    perm_infer,
                    InferenceVarData::lower_chains,
                    async |env, chain| require_chain_is_owned(env, &chain, &or_else).await,
                )
                .await
            }
            Predicate::Lent => {
                env.require_for_all_infer_bounds(
                    perm_infer,
                    InferenceVarData::upper_chains,
                    async |env, chain| require_chain_is_lent(env, &chain, &or_else).await,
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
    or_else: Arc<dyn OrElse<'db> + 'db>,
) {
    env.spawn(
        TaskDescription::RequireBoundsNotProvablyPredicate(infer, predicate),
        async move |env| match predicate {
            Predicate::Copy => {
                env.require_for_all_infer_bounds(
                    infer,
                    InferenceVarData::upper_chains,
                    async |env, chain| {
                        require_chain_isnt_provably_copy(env, &chain, &or_else).await
                    },
                )
                .await
            }
            Predicate::Move => {
                todo!()
            }
            Predicate::Owned => {
                todo!()
            }
            Predicate::Lent => {
                todo!()
            }
        },
    );
}

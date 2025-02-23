use dada_ir_ast::{diagnostic::Errors, span::Span};

use crate::{
    check::{
        env::Env,
        predicates::{
            Predicate,
            report::{report_infer_is_contradictory, report_var_must_be_but_is_not_declared_to_be},
        },
        report::{Because, OrElse},
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

use super::report::report_var_must_not_be_declared_but_is;

pub(crate) fn test_var_is_provably<'db>(
    env: &Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> bool {
    env.var_is_declared_to_be(var, predicate)
}

pub(super) fn require_var_is<'db>(
    env: &Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(or_else.report(env.db(), Because::VarNotDeclaredToBe(var, predicate)))
    }
}

pub(super) fn require_var_isnt<'db>(
    env: &Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if !env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(or_else.report(env.db(), Because::VarDeclaredToBe(var, predicate)))
    }
}

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub(super) fn require_infer_is<'db>(
    env: &Env<'db>,
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
    if let Some(isnt_span) = isnt_already {
        return Err(report_infer_is_contradictory(
            env, infer, predicate, span, isnt_span,
        ));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    env.runtime()
        .require_inference_var_is(infer, predicate, span);

    Ok(())
}

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub(super) fn require_infer_isnt<'db>(
    env: &Env<'db>,
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
    if let Some(is_span) = is_already {
        return Err(report_infer_is_contradictory(
            env, infer, predicate, is_span, span,
        ));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    env.runtime()
        .require_inference_var_isnt(infer, predicate, span);

    Ok(())
}

/// Wait until we know that the inference variable IS (or IS NOT) the given predicate.
pub(crate) async fn test_infer_is_known_to_be<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> bool {
    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let (is, isnt) = (
                data.is_known_to_provably_be(predicate),
                data.is_known_not_to_provably_be(predicate),
            );
            if is.is_some() {
                Some(true)
            } else if isnt.is_some() {
                Some(false)
            } else {
                None
            }
        })
        .await
}

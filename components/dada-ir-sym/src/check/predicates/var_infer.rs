use dada_ir_ast::{diagnostic::Errors, span::Span};

use crate::{
    check::{
        env::Env,
        predicates::{
            Predicate,
            report::{report_infer_is_contradictory, report_var_must_be_but_is_not_declared_to_be},
        },
    },
    ir::{indices::InferVarIndex, variables::SymVariable},
};

pub(crate) fn test_var_is<'db>(
    env: &Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> bool {
    env.var_is_declared_to_be(var, predicate)
}

pub(super) fn require_var_is<'db>(
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

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub(super) fn require_infer_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<()> {
    let inverted_predicate = predicate.invert();
    let (is_already, is_inverted_already) = env.runtime().with_inference_var_data(infer, |data| {
        (
            data.is_known_to_be(predicate),
            data.is_known_to_be(inverted_predicate),
        )
    });

    // Check if we are already required to be the predicate.
    if is_already.is_some() {
        return Ok(());
    }

    // Check if were already required to be the inverted predicate
    // and report an error if so.
    if let Some(inverted_span) = is_inverted_already {
        return Err(report_infer_is_contradictory(
            env,
            infer,
            predicate,
            span,
            inverted_predicate,
            inverted_span,
        ));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    env.runtime()
        .require_inference_var_is(infer, predicate, span);

    Ok(())
}

/// Wait until we know that the inference variable IS (or IS NOT) the given predicate.
pub(crate) async fn test_infer_is<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> bool {
    let inverted = predicate.invert();

    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let (is, is_inverted) = (
                data.is_known_to_be(predicate),
                data.is_known_to_be(inverted),
            );
            if is.is_some() {
                Some(true)
            } else if is_inverted.is_some() {
                Some(false)
            } else {
                None
            }
        })
        .await
}

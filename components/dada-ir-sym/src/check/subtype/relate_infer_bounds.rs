use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{env::Env, inference::Direction, live_places::LivePlaces, report},
    ir::{indices::InferVarIndex, types::SymPerm},
};

use super::terms::require_sub_terms;

/// A task that runs for each type inference variable. It awaits any upper/lower bounds
/// and propagates a corresponding bound.
pub async fn relate_infer_bounds<'db>(env: &mut Env<'db>, infer: InferVarIndex) -> Errors<()> {
    let mut lower_bound = None;
    let mut upper_bound = None;

    let mut bounds = env.term_bounds(SymPerm::my(env.db()), infer, None);
    while let Some((direction, new_bound)) = bounds.next(env).await {
        match direction {
            Direction::FromBelow => lower_bound = Some(new_bound),
            Direction::FromAbove => upper_bound = Some(new_bound),
        }

        if let (Some(lower), Some(upper)) = (lower_bound, upper_bound) {
            // FIXME: the iterator should be yielding up ArcOrElse values
            require_sub_terms(
                env,
                LivePlaces::infer_bounds(),
                lower,
                upper,
                &report::BadSubtermError::new(env.infer_var_span(infer), lower, upper),
            )
            .await?;
        }
    }

    Ok(())
}

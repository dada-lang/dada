use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        inference::Direction,
        live_places::LivePlaces,
        red::RedTy,
        report::{Because, OrElse, OrElseHelper},
        to_red::ToRedTy,
    },
    ir::types::{SymTy, SymTyName},
};

use super::terms::require_sub_terms;

/// Requires that `ty` resolves to a future type
/// that awaits a value of type `awaited_ty`.
pub async fn require_future_type<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    ty: SymTy<'db>,
    awaited_ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let (red_ty, _) = ty.to_red_ty(env);
    require_future_red_type(env, live_after, red_ty, awaited_ty, or_else).await
}

#[boxed_async_fn]
async fn require_future_red_type<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    red_ty: RedTy<'db>,
    awaited_ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match red_ty {
        RedTy::Error(reported) => Err(reported),

        RedTy::Named(sym_ty_name, generic_args) => match sym_ty_name {
            SymTyName::Future => {
                let future_ty_arg = generic_args[0].assert_type(db);
                require_sub_terms(
                    env,
                    live_after,
                    future_ty_arg.into(),
                    awaited_ty.into(),
                    or_else,
                )
                .await
            }
            SymTyName::Primitive(_) | SymTyName::Aggregate(_) | SymTyName::Tuple { arity: _ } => {
                Err(or_else.report(env, Because::JustSo))
            }
        },

        RedTy::Var(_) | RedTy::Never => Err(or_else.report(env, Because::JustSo)),

        RedTy::Infer(infer) => {
            // For inference variables: find the current lower bound
            // and check if it is numeric. Since the bound can only get tighter,
            // that is sufficient (indeed, numeric types have no subtypes).
            let Some((lower_red_ty, arc_or_else)) =
                env.red_ty_bound(infer, Direction::FromBelow).await
            else {
                return Err(
                    or_else.report(env, Because::UnconstrainedInfer(env.infer_var_span(infer)))
                );
            };
            require_future_red_type(
                env,
                live_after,
                lower_red_ty.clone(),
                awaited_ty,
                &or_else.map_because(move |_| {
                    Because::InferredLowerBound(lower_red_ty.clone(), arc_or_else.clone())
                }),
            )
            .await
        }

        RedTy::Perm => unreachable!("SymTy had a red ty of SymPerm"),
    }
}

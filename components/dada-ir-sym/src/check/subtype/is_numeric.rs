use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        predicates::{Predicate, var_infer::require_infer_is},
        red::RedTy,
        report::{Because, OrElse, OrElseHelper},
        to_red::ToRedTy,
    },
    ir::{
        primitive::SymPrimitiveKind,
        types::{SymTy, SymTyName},
    },
};

pub async fn require_numeric_type<'db>(
    env: &mut Env<'db>,
    ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let (red_ty, _) = ty.to_red_ty(env);
    require_numeric_red_type(env, red_ty, or_else).await
}

#[boxed_async_fn]
async fn require_numeric_red_type<'db>(
    env: &mut Env<'db>,
    red_ty: RedTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    match red_ty {
        RedTy::Error(reported) => Err(reported),
        RedTy::Named(sym_ty_name, _) => match sym_ty_name {
            SymTyName::Primitive(sym_primitive) => match sym_primitive.kind(db) {
                SymPrimitiveKind::Bool | SymPrimitiveKind::Char => {
                    Err(or_else.report(env, Because::JustSo))
                }
                SymPrimitiveKind::Int { bits: _ }
                | SymPrimitiveKind::Isize
                | SymPrimitiveKind::Uint { bits: _ }
                | SymPrimitiveKind::Usize
                | SymPrimitiveKind::Float { bits: _ } => Ok(()),
            },
            SymTyName::Aggregate(_) | SymTyName::Future | SymTyName::Tuple { arity: _ } => {
                Err(or_else.report(env, Because::JustSo))
            }
        },

        RedTy::Var(_) | RedTy::Never => Err(or_else.report(env, Because::JustSo)),

        RedTy::Infer(infer) => {
            env.require_both(
                async |env| require_infer_is(env, infer, Predicate::Copy, or_else),
                async |env| {
                    // For inference variables: find the current lower bound
                    // and check if it is numeric. Since the bound can only get tighter,
                    // that is sufficient (indeed, numeric types have no subtypes).
                    let Some((lower_red_ty, arc_or_else)) = env
                        .loop_on_inference_var(infer, |data| data.lower_red_ty())
                        .await
                    else {
                        return Err(or_else
                            .report(env, Because::UnconstrainedInfer(env.infer_var_span(infer))));
                    };
                    require_numeric_red_type(
                        env,
                        lower_red_ty.clone(),
                        &or_else.map_because(move |_| {
                            Because::InferredLowerBound(lower_red_ty.clone(), arc_or_else.clone())
                        }),
                    )
                    .await
                },
            )
            .await
        }

        RedTy::Perm => unreachable!("SymTy had a red ty of SymPerm"),
    }
}

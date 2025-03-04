use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::vecset::VecSet;

use crate::{
    check::{
        chains::RedTy,
        env::Env,
        report::{ArcOrElse, OrElse},
    },
    ir::{
        indices::{FromInfer, InferVarIndex},
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymTy},
    },
};

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    FromBelow,
    FromAbove,
}

/// Require that `lower <= ?X`.
pub async fn require_infer_has_lower_bound<'db>(
    env: &Env<'db>,
    bound: &RedTy<'db>,
    infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<RedTy<'db>> {
    require_infer_has_bound(env, Direction::FromBelow, bound, infer, or_else).await
}

/// Require that `?X <= upper`.
pub async fn require_infer_has_upper_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    bound: &RedTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<RedTy<'db>> {
    require_infer_has_bound(env, Direction::FromAbove, bound, infer, or_else).await
}

async fn require_infer_has_bound<'db>(
    env: &Env<'db>,
    direction: Direction,
    bound: &RedTy<'db>,
    infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<RedTy<'db>> {
    match bounding_red_ty(env, direction, infer) {
        None => {
            // Inference variable does not currently have a red-ty bound.
            // Create a generalized version of `bound` and use that.
            let span = env.infer_var_span(infer);
            let generalized = generalize(env, bound, span)?;
            set_bounding_red_ty(env, direction, infer, generalized.clone(), or_else);
            Ok(generalized)
        }

        Some((generalized, _generalized_or_else)) => {
            // There is already a red-ty bound on the inference variable.
            //
            // FIXME: We may need to adjust this bound once we introduce enum.
            Ok(generalized)
        }
    }
}

fn bounding_red_ty<'db>(
    env: &Env<'db>,
    direction: Direction,
    infer: InferVarIndex,
) -> Option<(RedTy<'db>, ArcOrElse<'db>)> {
    match direction {
        Direction::FromBelow => env
            .runtime()
            .with_inference_var_data(infer, |data| data.lower_red_ty().clone()),
        Direction::FromAbove => env
            .runtime()
            .with_inference_var_data(infer, |data| data.upper_red_ty().clone()),
    }
}

fn set_bounding_red_ty<'db>(
    env: &Env<'db>,
    direction: Direction,
    infer: InferVarIndex,
    red_ty: RedTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> ArcOrElse<'db> {
    match direction {
        Direction::FromBelow => env.runtime().set_lower_red_ty(infer, red_ty, or_else),
        Direction::FromAbove => env.runtime().set_upper_red_ty(infer, red_ty, or_else),
    }
}

fn generalize<'db>(env: &Env<'db>, red_ty: &RedTy<'db>, span: Span<'db>) -> Errors<RedTy<'db>> {
    let db = env.db();
    let red_ty_generalized = match red_ty {
        RedTy::Error(reported) => return Err(*reported),
        RedTy::Never => RedTy::Never,
        RedTy::Infer(_) => unreachable!("infer should not get here"),
        RedTy::Var(sym_variable) => RedTy::Var(*sym_variable),
        RedTy::Perm => RedTy::Perm,
        RedTy::Named(sym_ty_name, generics) => {
            let generics_generalized = generics
                .iter()
                .map(|generic| match *generic {
                    SymGenericTerm::Type(_) => {
                        let v = env.fresh_inference_var(SymGenericKind::Type, span);
                        SymTy::infer(db, v).into()
                    }
                    SymGenericTerm::Perm(_) => {
                        let v = env.fresh_inference_var(SymGenericKind::Perm, span);
                        SymPerm::infer(db, v).into()
                    }
                    SymGenericTerm::Place(p) => SymGenericTerm::Place(p),
                    SymGenericTerm::Error(reported) => SymGenericTerm::Error(reported),
                })
                .collect();
            RedTy::Named(*sym_ty_name, generics_generalized)
        }
    };
    Ok(red_ty_generalized)
}

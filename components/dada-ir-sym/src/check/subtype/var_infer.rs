use dada_ir_ast::{ast::VariableDecl, diagnostic::Errors, span::Span};
use dada_util::vecset::VecSet;

use crate::{
    check::{
        chains::{RedTerm, RedTy},
        combinator,
        env::Env,
        report::{Because, OrElse, OrElseHelper},
    },
    ir::{
        indices::{FromInfer, InferVarIndex},
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymTy, Variance},
    },
};

use super::terms::{require_sub_red_terms, require_sub_terms};

/// Require that `lower <: ?X`.
///
/// If `?X` does not yet have a lower bound,
/// then we set the lower bound to `generalized(lower)`.
///
/// The `generalized` operation will replace
///
/// If `?X` already has a lower bound,
/// then
async fn require_infer_has_lower_bound<'a, 'db>(
    env: &'a Env<'db>,
    lower_red_ty: RedTy<'db>,
    infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let Some((generalized_red_ty, _generalized_or_else)) = env
        .runtime()
        .with_inference_var_data(infer, |data| data.lower_red_ty().clone())
    else {
        let span = env
            .runtime()
            .with_inference_var_data(infer, |data| data.span());
        let (lower_generalized, relate_pairs) = generalize(env, lower_red_ty, span)?;
        let or_else = env
            .runtime()
            .set_lower_red_ty(infer, lower_generalized, or_else);

        for RelatePair {
            variance,
            original_term,
            generalized_term,
        } in relate_pairs
        {
            // We want `lower[$L0...$LN] <= generalized[$G0...$GN] <= ?X` -- for that to happen...
            match variance {
                Variance::Covariant => {
                    // Term `$Li` in `lower` must be less than term `$Gi` in `generalized`
                    require_sub_terms(env, original_term, generalized_term, &or_else).await?
                }
                Variance::Contravariant => {
                    // Reverse of above
                    require_sub_terms(env, generalized_term, original_term, &or_else).await?
                }
                Variance::Invariant => {
                    // Must be equal
                    combinator::require_both(
                        require_sub_terms(env, original_term, generalized_term, &or_else),
                        require_sub_terms(env, generalized_term, original_term, &or_else),
                    )
                    .await?
                }
            }
        }

        return Ok(());
    };

    // We want `lower[$L0...$LN] <= generalized[$G0...$GN] <= ?X` -- for that to happen...
    let db = env.db();
    let lower_red_term = RedTerm::new(db, VecSet::default(), lower_red_ty);
    let geeneralized_red_term = RedTerm::new(db, VecSet::default(), generalized_red_ty);
    require_sub_red_terms(env, lower_red_term, geeneralized_red_term, or_else).await?;

    Ok(())
}

struct RelatePair<'db> {
    variance: Variance,
    original_term: SymGenericTerm<'db>,
    generalized_term: SymGenericTerm<'db>,
}

fn generalize<'db>(
    env: &Env<'db>,
    red_ty: RedTy<'db>,
    span: Span<'db>,
) -> Errors<(RedTy<'db>, Vec<RelatePair<'db>>)> {
    let db = env.db();
    let mut relate_pairs = vec![];
    let red_ty_generalized = match red_ty {
        RedTy::Error(reported) => return Err(reported),
        RedTy::Never => RedTy::Never,
        RedTy::Infer(_) => unreachable!("infer should not get here"),
        RedTy::Var(sym_variable) => RedTy::Var(sym_variable),
        RedTy::Perm => RedTy::Perm,
        RedTy::Named(sym_ty_name, sym_generic_terms) => {
            let variances = env.variances(sym_ty_name);
            let generics_generalized = sym_generic_terms
                .iter()
                .copied()
                .zip(variances)
                .map(|(generic, variance)| match generic {
                    SymGenericTerm::Type(_) => {
                        let v = env.fresh_inference_var(SymGenericKind::Type, span);
                        relate_pairs.push(RelatePair {
                            variance,
                            original_term: generic,
                            generalized_term: SymTy::infer(db, v).into(),
                        });
                        SymTy::infer(db, v).into()
                    }
                    SymGenericTerm::Perm(_) => {
                        let v = env.fresh_inference_var(SymGenericKind::Perm, span);
                        relate_pairs.push(RelatePair {
                            variance,
                            original_term: generic,
                            generalized_term: SymPerm::infer(db, v).into(),
                        });
                        SymPerm::infer(db, v).into()
                    }
                    SymGenericTerm::Place(p) => SymGenericTerm::Place(p),
                    SymGenericTerm::Error(reported) => SymGenericTerm::Error(reported),
                })
                .collect();
            RedTy::Named(sym_ty_name, generics_generalized)
        }
    };
    Ok((red_ty_generalized, relate_pairs))
}

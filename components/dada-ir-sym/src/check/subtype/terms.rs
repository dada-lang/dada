//! Implement object-level subtyping.

use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{
        chains::{RedTerm, RedTy, ToRedTerm},
        combinator,
        env::Env,
        predicates::{
            is_provably_copy::term_is_provably_copy, is_provably_lent::term_is_provably_lent,
            is_provably_move::term_is_provably_move, is_provably_owned::term_is_provably_owned,
            isnt_provably_copy::term_isnt_provably_copy, require_copy::require_term_is_copy,
            require_isnt_provably_copy::require_term_isnt_provably_copy,
            require_lent::require_term_is_lent, require_move::require_term_is_move,
            require_owned::require_term_is_owned, require_term_is_leased, term_is_provably_leased,
        },
        report::{Because, OrElse},
    },
    ir::{
        classes::SymAggregateStyle,
        types::{SymGenericTerm, SymTy, SymTyKind},
    },
};

use super::chains::require_sub_red_perms;

pub async fn require_assignable_type<'db>(
    env: &Env<'db>,
    value_ty: SymTy<'db>,
    place_ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    match (value_ty.kind(db), place_ty.kind(db)) {
        (SymTyKind::Never, _) => Ok(()),
        _ => require_sub_terms(env, value_ty.into(), place_ty.into(), or_else).await,
    }
}

pub async fn require_sub_terms<'a, 'db>(
    env: &'a Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    combinator::require_all!(
        propagate_bounds(env, lower.into(), upper.into(), or_else),
        async {
            // Reduce and relate chains
            let red_term_lower = lower.to_red_term(db, env).await;
            let red_term_upper = upper.to_red_term(db, env).await;
            require_sub_red_terms(env, red_term_lower, red_term_upper, or_else).await
        },
    )
    .await
}

/// Whenever we require that `lower <: upper`, we can also propagate certain bounds,
/// such as copy/lent and owned/move, from lower-to-upper and upper-to-lower.
/// This can unblock inference.
async fn propagate_bounds<'db>(
    env: &Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    combinator::require_all!(
        // If subtype is copy, supertype must be
        async {
            if term_is_provably_copy(env, lower).await? {
                require_term_is_copy(env, upper, or_else).await?;
            }
            Ok(())
        },
        // If subtype is lent, supertype must be
        async {
            if term_is_provably_lent(env, lower).await? {
                require_term_is_lent(env, upper, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something move if you are move
        async {
            if term_is_provably_move(env, upper).await? {
                require_term_is_move(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something that isn't copy if you aren't copy
        async {
            if term_isnt_provably_copy(env, upper).await? {
                require_term_isnt_provably_copy(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something owned if you are owned
        async {
            if term_is_provably_owned(env, upper).await? {
                require_term_is_owned(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a supertype of something leased if you are leased
        async {
            if term_is_provably_leased(env, lower).await? {
                require_term_is_leased(env, upper, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something leased if you are leased
        async {
            if term_is_provably_leased(env, upper).await? {
                require_term_is_leased(env, lower, or_else).await?;
            }
            Ok(())
        },
    )
    .await
}

async fn require_sub_red_terms<'a, 'db>(
    env: &'a Env<'db>,
    lower: RedTerm<'db>,
    upper: RedTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match (lower.ty(), upper.ty()) {
        (&RedTy::Error(reported), _) | (_, &RedTy::Error(reported)) => Err(reported),

        (&RedTy::Infer(infer_lower), &RedTy::Infer(infer_upper)) => todo!(),
        (&RedTy::Infer(infer_lower), _) => todo!(),
        (_, &RedTy::Infer(infer_lower)) => todo!(),

        (
            &RedTy::Named(name_lower, ref lower_generics),
            &RedTy::Named(name_upper, ref upper_generics),
        ) => {
            if name_lower == name_upper {
                // relate generics
                // XXX

                match name_lower.style(env.db()) {
                    SymAggregateStyle::Struct => {}
                    SymAggregateStyle::Class => {
                        require_sub_red_perms(env, lower.chains(), upper.chains(), or_else).await?;
                    }
                }

                Ok(())
            } else {
                Err(or_else.report(env.db(), Because::NameMismatch(name_lower, name_upper)))
            }
        }
        (&RedTy::Named(..), _) | (_, &RedTy::Named(..)) => {
            Err(or_else.report(env.db(), Because::NotSubRedTys(lower, upper)))
        }

        (&RedTy::Never, &RedTy::Never) => {
            require_sub_red_perms(env, lower.chains(), upper.chains(), or_else).await
        }
        (&RedTy::Never, _) | (_, &RedTy::Never) => {
            Err(or_else.report(env.db(), Because::NotSubRedTys(lower, upper)))
        }

        (&RedTy::Var(var_lower), &RedTy::Var(var_upper)) => {
            if var_lower == var_upper {
                require_sub_red_perms(env, lower.chains(), upper.chains(), or_else).await
            } else {
                Err(or_else.report(env.db(), Because::UniversalMismatch(var_lower, var_upper)))
            }
        }
        (&RedTy::Var(_), _) | (_, &RedTy::Var(_)) => {
            Err(or_else.report(env.db(), Because::NotSubRedTys(lower, upper)))
        }

        (&RedTy::Perm, &RedTy::Perm) => {
            require_sub_red_perms(env, lower.chains(), upper.chains(), or_else).await
        }
    }
}

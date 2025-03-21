//! Implement object-level subtyping.

use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::{
            is_provably_copy::term_is_provably_copy, is_provably_lent::term_is_provably_lent,
            is_provably_move::term_is_provably_move, is_provably_owned::term_is_provably_owned,
            isnt_provably_copy::term_isnt_provably_copy, require_copy::require_term_is_copy,
            require_isnt_provably_copy::require_term_isnt_provably_copy,
            require_lent::require_term_is_lent, require_move::require_term_is_move,
            require_owned::require_term_is_owned, require_term_is_leased, term_is_provably_leased,
        },
        red::{Chain, RedTerm, RedTy},
        report::{Because, OrElse},
        subtype::chains::require_sub_red_perms,
        to_red::ToRedTerm,
    },
    ir::{
        classes::SymAggregateStyle,
        indices::{FromInfer, InferVarIndex},
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymTy, SymTyKind, SymTyName, Variance},
    },
};

pub async fn require_assignable_type<'db>(
    env: &mut Env<'db>,
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

pub async fn require_sub_terms<'db>(
    env: &mut Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.log("require_sub_terms", &[&lower, &upper]);
    env.require_both(
        async |env| propagate_bounds(env, lower, upper, or_else).await,
        async |env| {
            // Reduce and relate chains
            let red_term_lower = lower.to_red_term(env).await;
            let red_term_upper = upper.to_red_term(env).await;
            require_sub_red_terms(env, red_term_lower, red_term_upper, or_else).await
        },
    )
    .await
}

/// Whenever we require that `lower <: upper`, we can also propagate certain bounds,
/// such as copy/lent and owned/move, from lower-to-upper and upper-to-lower.
/// This can unblock inference.
async fn propagate_bounds<'db>(
    env: &mut Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.log("propagate_bounds", &[&lower, &upper]);
    env.require_all()
        .require(
            // If subtype is copy, supertype must be
            async |env| {
                if term_is_provably_copy(env, lower).await? {
                    require_term_is_copy(env, upper, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // If subtype is lent, supertype must be
            async |env| {
                if term_is_provably_lent(env, lower).await? {
                    require_term_is_lent(env, upper, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // Can only be a subtype of something move if you are move
            async |env| {
                if term_is_provably_move(env, upper).await? {
                    require_term_is_move(env, lower, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // Can only be a subtype of something that isn't copy if you aren't copy
            async |env| {
                if term_isnt_provably_copy(env, upper).await? {
                    require_term_isnt_provably_copy(env, lower, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // Can only be a subtype of something owned if you are owned
            async |env| {
                if term_is_provably_owned(env, upper).await? {
                    require_term_is_owned(env, lower, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // Can only be a supertype of something leased if you are leased
            async |env| {
                if term_is_provably_leased(env, lower).await? {
                    require_term_is_leased(env, upper, or_else).await?;
                }
                Ok(())
            },
        )
        .require(
            // Can only be a subtype of something leased if you are leased
            async |env| {
                if term_is_provably_leased(env, upper).await? {
                    require_term_is_leased(env, lower, or_else).await?;
                }
                Ok(())
            },
        )
        .finish()
        .await
}

#[boxed_async_fn]
pub async fn require_sub_red_terms<'db>(
    env: &mut Env<'db>,
    lower: RedTerm<'db>,
    upper: RedTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.log("require_sub_red_terms", &[&lower, &upper]);
    match (&lower.ty, &upper.ty) {
        (&RedTy::Error(reported), _) | (_, &RedTy::Error(reported)) => Err(reported),

        (&RedTy::Infer(lower_infer), &RedTy::Infer(upper_infer)) => {
            require_infer_sub_infer(env, lower, lower_infer, upper, upper_infer, or_else).await
        }

        (&RedTy::Infer(lower_infer), _) => {
            require_infer_sub_ty(env, lower.chains, lower_infer, upper, or_else).await
        }

        (_, &RedTy::Infer(upper_infer)) => {
            require_ty_sub_infer(env, lower, upper.chains, upper_infer, or_else).await
        }

        (
            &RedTy::Named(name_lower, ref lower_generics),
            &RedTy::Named(name_upper, ref upper_generics),
        ) => {
            if name_lower == name_upper {
                let variances = env.variances(name_lower);
                assert_eq!(lower_generics.len(), upper_generics.len());
                env.require_for_all(
                    variances
                        .iter()
                        .zip(lower_generics.iter().zip(upper_generics)),
                    async |env, (&variance, (&lower_generic, &upper_generic))| match variance {
                        Variance::Covariant => {
                            require_sub_terms(env, lower_generic, upper_generic, or_else).await
                        }
                        Variance::Contravariant => {
                            require_sub_terms(env, upper_generic, lower_generic, or_else).await
                        }
                        Variance::Invariant => {
                            env.require_both(
                                async |env| {
                                    require_sub_terms(env, lower_generic, upper_generic, or_else)
                                        .await
                                },
                                async |env| {
                                    require_sub_terms(env, upper_generic, lower_generic, or_else)
                                        .await
                                },
                            )
                            .await
                        }
                    },
                )
                .await?;

                match name_lower.style(env.db()) {
                    SymAggregateStyle::Struct => {}
                    SymAggregateStyle::Class => {
                        require_sub_red_perms(env, &lower.chains, &upper.chains, or_else).await?;
                    }
                }

                Ok(())
            } else {
                Err(or_else.report(env, Because::NameMismatch(name_lower, name_upper)))
            }
        }
        (&RedTy::Named(..), _) | (_, &RedTy::Named(..)) => {
            Err(or_else.report(env, Because::JustSo))
        }

        (&RedTy::Never, &RedTy::Never) => {
            require_sub_red_perms(env, &lower.chains, &upper.chains, or_else).await
        }
        (&RedTy::Never, _) | (_, &RedTy::Never) => Err(or_else.report(env, Because::JustSo)),

        (&RedTy::Var(var_lower), &RedTy::Var(var_upper)) => {
            if var_lower == var_upper {
                require_sub_red_perms(env, &lower.chains, &upper.chains, or_else).await
            } else {
                Err(or_else.report(env, Because::UniversalMismatch(var_lower, var_upper)))
            }
        }
        (&RedTy::Var(_), _) | (_, &RedTy::Var(_)) => Err(or_else.report(env, Because::JustSo)),

        (&RedTy::Perm, &RedTy::Perm) => {
            require_sub_red_perms(env, &lower.chains, &upper.chains, or_else).await
        }
    }
}

/// Require that `lower <: upper`, where both are type inference variables.
/// This will insert record `upper` as an upper bound of `lower`.
/// If `upper` is a new upper bound, it will begin looping,
/// taking each lower bound of `lower` and propagating it to `upper`
/// (in this case it will not return).
async fn require_infer_sub_infer<'db>(
    env: &mut Env<'db>,
    lower: RedTerm<'db>,
    lower_infer: InferVarIndex,
    upper: RedTerm<'db>,
    upper_infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    debug_assert_eq!(env.infer_var_kind(lower_infer), InferVarKind::Type);
    debug_assert_eq!(env.infer_var_kind(upper_infer), InferVarKind::Type);

    if lower_infer == upper_infer {
        return Ok(());
    }

    if env.insert_sub_infer_var_pair(lower_infer, upper_infer) {
        env.require_both(
            async |env| {
                env.for_each_bound(
                    Direction::FromBelow,
                    lower_infer,
                    async |env, lower_bound, _or_else| {
                        require_sub_red_terms(
                            env,
                            RedTerm::new(db, lower.chains.clone(), lower_bound.clone()),
                            upper.clone(),
                            or_else,
                        )
                        .await
                    },
                )
                .await
            },
            async |env| {
                env.for_each_bound(
                    Direction::FromAbove,
                    upper_infer,
                    async |env, upper_bound, _or_else| {
                        require_sub_red_terms(
                            env,
                            lower.clone(),
                            RedTerm::new(db, upper.chains.clone(), upper_bound.clone()),
                            or_else,
                        )
                        .await
                    },
                )
                .await
            },
        )
        .await
    } else {
        Ok(())
    }
}

/// Relate `lower_term` (not)
async fn require_ty_sub_infer<'db>(
    env: &mut Env<'db>,
    lower_term: RedTerm<'db>,
    upper_chains: VecSet<Chain<'db>>,
    upper_infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    debug_assert!(
        !matches!(lower_term.ty, RedTy::Infer(_)),
        "unexpected inference variable"
    );

    // Get the lower bounding red-ty from `upper_infer`;
    // if it doesn't have one yet, generalize `lower_term.ty` to create one.
    let generalized_ty = require_infer_has_bound(
        env,
        Direction::FromBelow,
        &lower_term.ty,
        upper_infer,
        or_else,
    )
    .await?;

    // Relate the lower term to the upper term
    require_sub_red_terms(
        env,
        lower_term,
        RedTerm::new(db, upper_chains, generalized_ty),
        or_else,
    )
    .await
}

/// Return the red-ty lower bound from `infer`, creating one if needed by generalizing `bound`.
/// Does not relate the return value and `bound` in any other way.
async fn require_infer_sub_ty<'db>(
    env: &mut Env<'db>,
    lower_chains: VecSet<Chain<'db>>,
    lower_infer: InferVarIndex,
    upper_term: RedTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    debug_assert!(
        !matches!(upper_term.ty, RedTy::Infer(_)),
        "unexpected inference variable"
    );

    // Get the upper bounding red-ty from `upper_infer`;
    // if it doesn't have one yet, generalize `upper_term.ty` to create one.
    let generalized_ty = require_infer_has_bound(
        env,
        Direction::FromAbove,
        &upper_term.ty,
        lower_infer,
        or_else,
    )
    .await?;

    require_sub_red_terms(
        env,
        RedTerm::new(db, lower_chains, generalized_ty),
        upper_term,
        or_else,
    )
    .await
}

/// Return the upper or lower (depending on direction) red-ty-bound from `infer`.
/// If `infer` does not yet have a suitable bound, create one by generalizing `bound`.
async fn require_infer_has_bound<'db>(
    env: &mut Env<'db>,
    direction: Direction,
    bound: &RedTy<'db>,
    infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<RedTy<'db>> {
    match env.red_ty_bound(infer, direction).peek() {
        None => {
            // Inference variable does not currently have a red-ty bound.
            // Create a generalized version of `bound` and use that.
            let span = env.infer_var_span(infer);
            let generalized = generalize(env, bound, span)?;
            env.red_ty_bound(infer, direction)
                .set(generalized.clone(), or_else);
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

/// *Generalize* returns a new red-ty created by replacing any generic arguments in `red_ty`
/// with fresh inference variables.
fn generalize<'db>(env: &mut Env<'db>, red_ty: &RedTy<'db>, span: Span<'db>) -> Errors<RedTy<'db>> {
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

/// A task that runs for each type inference variable. It awaits any upper/lower bounds
/// and propagates a corresponding bound.
pub async fn reconcile_ty_bounds<'db>(env: &mut Env<'db>, infer: InferVarIndex) -> Errors<()> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);
    let db = env.db();

    // Various views onto this inference variable
    let span = env.infer_var_span(infer);
    let perm_infer = env.perm_infer(infer);
    let red_chains = VecSet::from(Chain::infer(db, perm_infer));
    let red_term_with_perm_infer = |red_ty| RedTerm::new(db, red_chains.clone(), red_ty);

    env.require_both(
        async |env| {
            // For each type `T` where `?X <: T`...
            env.for_each_bound(Direction::FromAbove, infer, async |env, red_ty, or_else| {
                // see if that implies that `U <: ?X` for some `U`
                let lower_bound = match red_ty {
                    RedTy::Error(_) => None,

                    RedTy::Named(sym_ty_name, _) => match sym_ty_name {
                        SymTyName::Primitive(_) | SymTyName::Future | SymTyName::Tuple { .. } => {
                            Some(generalize(env, red_ty, span)?)
                        }
                        SymTyName::Aggregate(_sym_aggregate) => {
                            // FIXME(#241): check if `sym_aggregate` is an enum
                            // in which case we need to adjust
                            Some(generalize(env, red_ty, span)?)
                        }
                    },

                    RedTy::Never | RedTy::Var(..) => Some(red_ty.clone()),

                    RedTy::Infer(..) | RedTy::Perm => {
                        unreachable!("unexpected kind for red-ty bound: {red_ty:?}")
                    }
                };

                if let Some(lower_bound) = lower_bound {
                    require_ty_sub_infer(
                        env,
                        red_term_with_perm_infer(lower_bound),
                        red_chains.clone(),
                        infer,
                        &or_else,
                    )
                    .await?;
                }

                Ok(())
            })
            .await
        },
        async |env| {
            // For each type `T` where `T <: ?X`...
            env.for_each_bound(Direction::FromBelow, infer, async |env, red_ty, or_else| {
                // see if that implies that `?X <: U` for some `U`
                let upper_bound = match red_ty {
                    RedTy::Error(_) => None,

                    RedTy::Named(sym_ty_name, _) => match sym_ty_name {
                        SymTyName::Primitive(_) | SymTyName::Future | SymTyName::Tuple { .. } => {
                            Some(generalize(env, red_ty, span)?)
                        }
                        SymTyName::Aggregate(_sym_aggregate) => {
                            // FIXME(#241): check if `sym_aggregate` is an enum
                            // in which case we need to adjust
                            Some(generalize(env, red_ty, span)?)
                        }
                    },

                    RedTy::Never | RedTy::Var(..) => Some(red_ty.clone()),

                    RedTy::Infer(..) | RedTy::Perm => {
                        unreachable!("unexpected kind for red-ty bound: {red_ty:?}")
                    }
                };

                if let Some(upper_bound) = upper_bound {
                    require_infer_sub_ty(
                        env,
                        red_chains.clone(),
                        infer,
                        red_term_with_perm_infer(upper_bound),
                        &or_else,
                    )
                    .await?;
                }

                Ok(())
            })
            .await
        },
    )
    .await
}

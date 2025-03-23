//! Implement object-level subtyping.

use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        env::Env,
        inference::{Direction, InferVarKind},
        predicates::{
            Predicate, is_provably_copy::term_is_provably_copy,
            is_provably_lent::term_is_provably_lent, is_provably_move::term_is_provably_move,
            is_provably_owned::term_is_provably_owned, isnt_provably_copy::term_isnt_provably_copy,
            require_copy::require_term_is_copy,
            require_isnt_provably_copy::require_term_isnt_provably_copy,
            require_lent::require_term_is_lent, require_move::require_term_is_move,
            require_owned::require_term_is_owned, require_term_is_leased, term_is_provably_leased,
            var_infer::require_infer_is,
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

    env.require_all()
        .require(async |env| propagate_inverse_bound(env, infer, Direction::FromAbove).await)
        .require(async |env| propagate_inverse_bound(env, infer, Direction::FromBelow).await)
        .require(async |env| propagate_predicates_from_below(env, infer).await)
        .require(async |env| propagate_predicates_from_above(env, infer).await)
        .finish()
        .await
}

async fn propagate_predicates_from_below<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
) -> Errors<()> {
    let db = env.db();

    // Iterate over each lower bound `LB <: X`...
    env.for_each_bound(
        Direction::FromBelow,
        infer,
        async |env, red_ty, or_else| match red_ty {
            RedTy::Error(_) => Ok(()),
            RedTy::Named(sym_ty_name, sym_generic_terms) => match sym_ty_name {
                SymTyName::Primitive(_) => require_infer_is(env, infer, Predicate::Copy, &or_else),
                SymTyName::Aggregate(sym_aggregate) if sym_aggregate.is_struct(db) => {

                }
                SymTyName::Future => Ok(())
                SymTyName::Tuple { arity } => todo!(),
            },
            RedTy::Never => Ok(()),
            RedTy::Var(sym_variable) => Ok(()),

            RedTy::Infer(..) | RedTy::Perm => {
                unreachable!("unexpected kind for red-ty bound: {red_ty:?}")
            }
        },
    )
    .await
}

async fn propagate_predicates_from_above<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
) -> Errors<()> {
    let db = env.db();

    // Iterate over each lower bound `LB <: X`...
    env.for_each_bound(Direction::FromAbove, infer, async |env, red_ty, or_else| {})
        .await
}

/// What each upper or lower  (depending on `direction`) bound `B` that is added to `infer`
/// and add a lower or upper (respectively) bound `B1` that is implied by `B`.
///
/// # Example
///
/// Given `Direction::FromAbove`, if we learn that a new upper bound `i32`,
/// i.e., that `infer <: i32`, then this also implies `i32 <: infer`, so we add the
/// lower bound `i32`.
///
/// Given `Direction::FromAbove`, if we learn that a new upper bound `Vec[T1]`,
/// i.e., that `infer <: Vec[T1]`, then this also implies `Vec[_] <: infer`, so we add the
/// lower bound `Vec[?X]` for a fresh `?X`. This will in turn require that `?X <: T1`.
///
/// NB. In some cases (e.g., `Vec[i32]` for sure...) we could avoid creating the inference
/// variable but right now we just *always* create one since I didn't want to think about it.
async fn propagate_inverse_bound<'db>(
    env: &mut Env<'db>,
    infer: InferVarIndex,
    direction: Direction,
) -> Errors<()> {
    let db = env.db();

    let span = env.infer_var_span(infer);

    // Every type inference variable has both an associated
    // permission variable `perm_infer` and upper/lower red-ty
    // bounds (see `InferenceVarBounds` in the `inference` module
    // for more details).
    //
    // The bound `B` and opposite bound `B1` that we are creating
    // correspond to the red-ty bound part of `?X`.
    // To relate these bounds fully to `?X` will will need to combine
    // them with the permissions from `perm_infer`.
    // This is because you can't directly subtype e.g. a struct
    // without knowing the permission.
    let perm_infer = env.perm_infer(infer);
    let red_chains = VecSet::from(Chain::infer(db, perm_infer));
    let red_term_with_perm_infer = |red_ty| RedTerm::new(db, red_chains.clone(), red_ty);

    // For each new bound `B` where `?X <: B`...
    //
    // NB: Comments are written assuming `Direction::FromAbove`.
    env.for_each_bound(direction, infer, async |env, red_ty, or_else| {
        // ...see if that implies an opposite bound `B1 <: ?X`...
        let opposite_bound = match red_ty {
            RedTy::Error(_) => None,

            RedTy::Named(sym_ty_name, _) => match sym_ty_name {
                SymTyName::Primitive(_) | SymTyName::Future | SymTyName::Tuple { .. } => {
                    Some(generalize(env, red_ty, span)?)
                }
                SymTyName::Aggregate(_sym_aggregate) => {
                    // FIXME(#241): check if `sym_aggregate` is an enum
                    // in which case we need to adjust based on `direction`
                    Some(generalize(env, red_ty, span)?)
                }
            },

            RedTy::Never | RedTy::Var(..) => Some(red_ty.clone()),

            RedTy::Infer(..) | RedTy::Perm => {
                unreachable!("unexpected kind for red-ty bound: {red_ty:?}")
            }
        };

        // If so, add the new opposite bound `B1`.
        if let Some(opposite_bound) = opposite_bound {
            match direction {
                Direction::FromAbove => {
                    // ... `?X <: B` so `B1 <: ?X`
                    require_ty_sub_infer(
                        env,
                        // Combine `B1` with the permission variable from `?X`
                        red_term_with_perm_infer(opposite_bound),
                        // Pass `?X` along with its permisson variable as the upper term
                        red_chains.clone(),
                        infer,
                        &or_else,
                    )
                    .await?;
                }
                Direction::FromBelow => {
                    // ... `B <: ?X` so `?X <: B1`
                    require_infer_sub_ty(
                        env,
                        // Pass `?X` along with its permisson variable as the lower term
                        red_chains.clone(),
                        infer,
                        // Combine `B1` with the permission variable from `?X`
                        red_term_with_perm_infer(opposite_bound),
                        &or_else,
                    )
                    .await?
                }
            }
        }

        Ok(())
    })
    .await
}

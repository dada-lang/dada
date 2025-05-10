//! Implement object-level subtyping.

use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        inference::{Direction, InferVarKind},
        live_places::LivePlaces,
        red::RedTy,
        report::{Because, OrElse},
        to_red::ToRedTy,
    },
    ir::{
        classes::SymAggregateStyle,
        indices::{FromInfer, InferVarIndex},
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymTy, SymTyKind, SymTyName, Variance},
        variables,
    },
};

use super::perms::require_sub_opt_perms;

pub async fn require_assignable_type<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    value_ty: SymTy<'db>,
    place_ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    match (value_ty.kind(db), place_ty.kind(db)) {
        (SymTyKind::Never, _) => Ok(()),
        _ => require_sub_terms(env, live_after, value_ty.into(), place_ty.into(), or_else).await,
    }
}

pub async fn require_sub_terms<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.log("require_sub_terms", &[&lower, &upper]);
    let red_term_lower = lower.to_red_ty(env);
    let red_term_upper = upper.to_red_ty(env);
    require_sub_red_terms(env, live_after, red_term_lower, red_term_upper, or_else).await
}

#[boxed_async_fn]
pub async fn require_sub_red_terms<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    (lower_red_ty, lower_perm): (RedTy<'db>, Option<SymPerm<'db>>),
    (upper_red_ty, upper_perm): (RedTy<'db>, Option<SymPerm<'db>>),
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.log(
        "require_sub_red_terms",
        &[&lower_perm, &lower_red_ty, &upper_perm, &upper_red_ty],
    );
    match (&lower_red_ty, &upper_red_ty) {
        (&RedTy::Error(reported), _) | (_, &RedTy::Error(reported)) => Err(reported),

        (&RedTy::Infer(lower_infer), &RedTy::Infer(upper_infer)) => {
            require_infer_sub_infer(
                env,
                live_after,
                lower_perm,
                lower_infer,
                upper_perm,
                upper_infer,
                or_else,
            )
            .await
        }

        (&RedTy::Infer(lower_infer), _) => {
            require_infer_sub_ty(
                env,
                live_after,
                lower_perm,
                lower_infer,
                upper_perm,
                upper_red_ty,
                or_else,
            )
            .await
        }

        (_, &RedTy::Infer(upper_infer)) => {
            require_ty_sub_infer(
                env,
                live_after,
                lower_perm,
                lower_red_ty,
                upper_perm,
                upper_infer,
                or_else,
            )
            .await
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
                    async |env, (&variance, (&lower_generic, &upper_generic))| {
                        let db = env.db();
                        let mut lower_generic = lower_generic;
                        let mut upper_generic = upper_generic;

                        if !variance.relative {
                            if let Some(lower_perm) = lower_perm {
                                lower_generic = lower_perm.apply_to_term(db, lower_generic);
                            }

                            if let Some(upper_perm) = upper_perm {
                                upper_generic = upper_perm.apply_to_term(db, upper_generic);
                            }
                        }

                        env.require_both(
                            async |env| {
                                if variance.at_least_covariant {
                                    require_sub_terms(
                                        env,
                                        live_after,
                                        lower_generic,
                                        upper_generic,
                                        or_else,
                                    )
                                    .await
                                } else {
                                    Ok(())
                                }
                            },
                            async |env| {
                                if variance.at_least_contravariant {
                                    require_sub_terms(
                                        env,
                                        live_after,
                                        upper_generic,
                                        lower_generic,
                                        or_else,
                                    )
                                    .await
                                } else {
                                    Ok(())
                                }
                            },
                        )
                        .await
                    },
                )
                .await?;

                match name_lower.style(env.db()) {
                    SymAggregateStyle::Struct => {}
                    SymAggregateStyle::Class => {
                        require_sub_opt_perms(env, live_after, lower_perm, upper_perm, or_else)
                            .await?;
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
            require_sub_opt_perms(env, live_after, lower_perm, upper_perm, or_else).await
        }
        (&RedTy::Never, _) | (_, &RedTy::Never) => Err(or_else.report(env, Because::JustSo)),

        (&RedTy::Var(var_lower), &RedTy::Var(var_upper)) => {
            if var_lower == var_upper {
                require_sub_opt_perms(env, live_after, lower_perm, upper_perm, or_else).await
            } else {
                Err(or_else.report(env, Because::UniversalMismatch(var_lower, var_upper)))
            }
        }
        (&RedTy::Var(_), _) | (_, &RedTy::Var(_)) => Err(or_else.report(env, Because::JustSo)),

        (&RedTy::Perm, &RedTy::Perm) => {
            require_sub_opt_perms(env, live_after, lower_perm, upper_perm, or_else).await
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
    live_after: LivePlaces,
    lower_perm: Option<SymPerm<'db>>,
    lower_infer: InferVarIndex,
    upper_perm: Option<SymPerm<'db>>,
    upper_infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    debug_assert_eq!(env.infer_var_kind(lower_infer), InferVarKind::Type);
    debug_assert_eq!(env.infer_var_kind(upper_infer), InferVarKind::Type);

    if lower_infer == upper_infer {
        return Ok(());
    }

    // FIXME: needs to take live-places into account
    if env.insert_sub_infer_var_pair(lower_infer, upper_infer) {
        env.require_both(
            async |env| {
                env.for_each_bound(
                    Direction::FromBelow,
                    lower_infer,
                    async |env, lower_bound, _or_else| {
                        require_sub_red_terms(
                            env,
                            live_after,
                            (lower_bound.clone(), lower_perm),
                            (RedTy::Infer(upper_infer), upper_perm),
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
                            live_after,
                            (RedTy::Infer(lower_infer), lower_perm),
                            (upper_bound.clone(), upper_perm),
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
    live_after: LivePlaces,
    lower_perm: Option<SymPerm<'db>>,
    lower_ty: RedTy<'db>,
    upper_perm: Option<SymPerm<'db>>,
    upper_infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    debug_assert!(
        !matches!(lower_ty, RedTy::Infer(_)),
        "unexpected inference variable"
    );

    // Get the lower bounding red-ty from `upper_infer`;
    // if it doesn't have one yet, generalize `lower_ty` to create one.
    let generalized_ty =
        require_infer_has_bound(env, Direction::FromBelow, &lower_ty, upper_infer, or_else).await?;

    // Relate the lower term to the upper term
    require_sub_red_terms(
        env,
        live_after,
        (lower_ty, lower_perm),
        (generalized_ty, upper_perm),
        or_else,
    )
    .await
}

/// Return the red-ty lower bound from `infer`, creating one if needed by generalizing `bound`.
/// Does not relate the return value and `bound` in any other way.
async fn require_infer_sub_ty<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_perm: Option<SymPerm<'db>>,
    lower_infer: InferVarIndex,
    upper_perm: Option<SymPerm<'db>>,
    upper_ty: RedTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    debug_assert!(
        !matches!(upper_ty, RedTy::Infer(_)),
        "unexpected inference variable"
    );

    // Get the upper bounding red-ty from `upper_infer`;
    // if it doesn't have one yet, generalize `upper_term.ty` to create one.
    let generalized_ty =
        require_infer_has_bound(env, Direction::FromAbove, &upper_ty, lower_infer, or_else).await?;

    require_sub_red_terms(
        env,
        live_after,
        (generalized_ty, lower_perm),
        (upper_ty, upper_perm),
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
    match env.red_bound(infer, direction).peek_ty() {
        None => {
            // Inference variable does not currently have a red-ty bound.
            // Create a generalized version of `bound` and use that.
            let span = env.infer_var_span(infer);
            let generalized = generalize(env, bound, span)?;
            env.red_bound(infer, direction)
                .set_ty(generalized.clone(), or_else);
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
#[expect(clippy::needless_lifetimes)]
pub async fn reconcile_ty_bounds<'db>(env: &mut Env<'db>, infer: InferVarIndex) -> Errors<()> {
    assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);

    env.require_all()
        .require(async |env| propagate_inverse_bound(env, infer, Direction::FromAbove).await)
        .require(async |env| propagate_inverse_bound(env, infer, Direction::FromBelow).await)
        .finish()
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
#[expect(clippy::needless_lifetimes)]
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
    let perm_infer = SymPerm::infer(db, env.perm_infer(infer));

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
                        LivePlaces::fixme(),
                        // Combine `B1` with the permission variable from `?X`
                        Some(perm_infer),
                        opposite_bound,
                        // Pass `?X` along with its permission variable as the upper term
                        Some(perm_infer),
                        infer,
                        &or_else,
                    )
                    .await?;
                }
                Direction::FromBelow => {
                    // ... `B <: ?X` so `?X <: B1`
                    require_infer_sub_ty(
                        env,
                        LivePlaces::fixme(),
                        // Pass `?X` along with its permission variable as the lower term
                        Some(perm_infer),
                        infer,
                        // Combine `B1` with the permission variable from `?X`
                        Some(perm_infer),
                        opposite_bound,
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

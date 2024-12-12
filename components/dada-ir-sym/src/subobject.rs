//! Implement object-level subtyping.

use std::future::Future;

use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use futures::StreamExt;

use crate::{
    bound::Direction,
    check::env::Env,
    ir::indices::InferVarIndex,
    ir::primitive::SymPrimitiveKind,
    ir::types::{SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyKind, SymTyName},
    ir::variables::SymVariable,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Expected {
    // The lower type is the expected one.
    Lower,

    // The upper type is the expected one.
    Upper,
}
impl Expected {
    fn expected_found<T>(self, lower: T, upper: T) -> (T, T) {
        match self {
            Expected::Lower => (lower, upper),
            Expected::Upper => (upper, lower),
        }
    }
}

pub async fn require_assignable_type<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    value_ty: SymTy<'db>,
    place_ty: SymTy<'db>,
) -> Errors<()> {
    let db = env.db();

    match (value_ty.kind(db), place_ty.kind(db)) {
        (SymTyKind::Never, _) => Ok(()),
        _ => require_subtype(env, Expected::Upper, span, value_ty, place_ty).await,
    }
}

pub fn require_subtype<'a, 'db>(
    env: &'a Env<'db>,

    // Tracks which of the types is the "expected" type from a user's point-of-view.
    // This typically starts out as `UpperBoundedBy`, meaning that the `sup` type is
    // the one that is "expected", but it can flip due to variance.
    expected: Expected,

    // Span that is forcing the sub-object comparison. We should probably track a more
    // complex cause.
    span: Span<'db>,

    // Prospective subtype
    lower: SymTy<'db>,

    // Prospective supertype
    upper: SymTy<'db>,
) -> impl Future<Output = Errors<()>> + use<'a, 'db> {
    Box::pin(async move {
        let db = env.db();

        match (lower.kind(db), upper.kind(db)) {
            (SymTyKind::Error(_), _) | (_, SymTyKind::Error(_)) => Ok(()),

            (SymTyKind::Var(univ_lower), SymTyKind::Var(univ_upper)) => {
                if univ_lower == univ_upper {
                    Ok(())
                } else {
                    Err(report_universal_mismatch(
                        env,
                        expected,
                        span,
                        *univ_lower,
                        *univ_upper,
                    ))
                }
            }

            (&SymTyKind::Infer(infer_var), _) => {
                bound_inference_var(
                    env,
                    span,
                    infer_var,
                    expected == Expected::Lower,
                    Direction::UpperBoundedBy,
                    upper.into(),
                )
                .await
            }

            (_, &SymTyKind::Infer(infer_var)) => {
                bound_inference_var(
                    env,
                    span,
                    infer_var,
                    expected == Expected::Upper,
                    Direction::LowerBoundedBy,
                    lower.into(),
                )
                .await
            }

            (
                SymTyKind::Named(name_lower, args_lower),
                SymTyKind::Named(name_upper, args_upper),
            ) => {
                if name_lower != name_upper {
                    return Err(report_class_name_mismatch(
                        env,
                        expected,
                        span,
                        *name_lower,
                        *name_upper,
                    ));
                }

                assert_eq!(args_lower.len(), args_upper.len());

                // FIXME: variance
                for (&arg_lower, &arg_upper) in args_lower.iter().zip(args_upper) {
                    require_sub_generic_term(env, expected, span, arg_lower, arg_upper).await?;
                }

                Ok(())
            }

            _ => {
                // FIXME
                Ok(())
            }
        }
    })
}

/// Introduce `term` as a lower or upper bound on `infer_var` (depending on `direction`).
/// This will also relate `term` to any previously added bounds, per the MLsub algorithm.
/// For example adding `term` as a lower bound will relate `term` to any previous upper bounds.
fn bound_inference_var<'a, 'db>(
    env: &'a Env<'db>,

    // Span that is forcing the sub-object comparison
    span: Span<'db>,

    // The inference variable to bound
    infer_var: InferVarIndex,

    // True if the inference variable is the expected type
    infer_var_expected: bool,

    // The relation of `term` to `infer_var`.
    direction: Direction,

    // The term to bound the inference variable by
    term: SymGenericTerm<'db>,
) -> impl Future<Output = Errors<()>> + use<'a, 'db> {
    Box::pin(async move {
        // If this variable already has the given bound, stop.
        if !env
            .runtime()
            .insert_inference_var_bound(infer_var, direction, term)
        {
            return Ok(());
        }

        // Relate `term` to existing bounds in the opposite direction.
        // For example, if we are adding a lower bound (i.e., `term <: ?X`),
        // then we get each existing bound `B` where `?X <: B` and require `term <: B`.
        let opposite_bounds: Vec<SymGenericTerm<'db>> =
            env.runtime().with_inference_var_data(infer_var, |data| {
                direction.reverse().infer_var_bounds(data).to_vec()
            });
        for opposite_bound in opposite_bounds {
            let (arg_sub, arg_sup, expected) = match direction {
                Direction::LowerBoundedBy => {
                    // If direction == LowerBounds, we are adding a new `T <: ?X`
                    // and we already knew that `?X <: opposite_bound`.
                    // Therefore we now require that `T <: opposite_bound`.
                    (
                        term,
                        opposite_bound,
                        if infer_var_expected {
                            // The inference variable is being replaced by opposite-bound
                            // in the position of upper-bound.
                            Expected::Upper
                        } else {
                            Expected::Lower
                        },
                    )
                }

                Direction::UpperBoundedBy => {
                    // Like the other match arm, but in reverse:
                    // We already knew that `opposite_bound <: ?X` and we are adding `?X <: T`.
                    // Therefore we now require that `opposite_bound <: T`.
                    (
                        opposite_bound,
                        term,
                        if infer_var_expected {
                            Expected::Lower
                        } else {
                            Expected::Upper
                        },
                    )
                }
            };

            require_sub_generic_term(env, expected, span, arg_sub, arg_sup).await?;
        }

        Ok(())
    })
}

async fn require_sub_generic_term<'db>(
    env: &Env<'db>,
    expected: Expected,
    span: Span<'db>,
    arg_lower: SymGenericTerm<'db>,
    arg_upper: SymGenericTerm<'db>,
) -> Errors<()> {
    match (arg_lower, arg_upper) {
        (SymGenericTerm::Type(lower), SymGenericTerm::Type(upper)) => {
            require_subtype(env, expected, span, lower, upper).await
        }
        (SymGenericTerm::Perm(lower), SymGenericTerm::Perm(upper)) => {
            require_subperms(env, span, lower, upper).await
        }
        (SymGenericTerm::Place(lower), SymGenericTerm::Place(upper)) => {
            require_subplaces(env, span, lower, upper).await
        }
        (SymGenericTerm::Error(_), _) => Ok(()),
        (_, SymGenericTerm::Error(_)) => Ok(()),
        _ => unreachable!("kind mismatch"),
    }
}

async fn require_subperms<'db>(
    _env: &Env<'db>,
    _span: Span<'db>,
    _lower: SymPerm<'db>,
    _upper: SymPerm<'db>,
) -> Errors<()> {
    todo!()
}

async fn require_subplaces<'db>(
    _env: &Env<'db>,
    _span: Span<'db>,
    _lower: SymPlace<'db>,
    _upper: SymPlace<'db>,
) -> Errors<()> {
    todo!()
}

pub async fn require_numeric_type<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    start_ty: SymTy<'db>,
) -> Errors<()> {
    let db = env.db();

    let mut bounds = env.transitive_upper_bounds(start_ty);
    while let Some(ty) = bounds.next().await {
        match *ty.kind(db) {
            SymTyKind::Error(_) => {}
            SymTyKind::Never => {}
            SymTyKind::Named(name, _) => match name {
                SymTyName::Primitive(prim) => match prim.kind(db) {
                    SymPrimitiveKind::Int { .. }
                    | SymPrimitiveKind::Isize
                    | SymPrimitiveKind::Uint { .. }
                    | SymPrimitiveKind::Usize
                    | SymPrimitiveKind::Float { .. } => {}
                    SymPrimitiveKind::Bool | SymPrimitiveKind::Char => {
                        return Err(report_numeric_type_expected(env, span, ty))
                    }
                },
                SymTyName::Future | SymTyName::Aggregate(_) | SymTyName::Tuple { .. } => {
                    return Err(report_numeric_type_expected(env, span, ty))
                }
            },
            SymTyKind::Var(_) => return Err(report_numeric_type_expected(env, span, ty)),
            SymTyKind::Infer(_) => {}
            SymTyKind::Perm(_, sym_ty) => {
                env.defer(span, async move |env| {
                    let _ = require_numeric_type(&env, span, sym_ty).await;
                });
            }
        }
    }

    Ok(())
}

fn report_class_name_mismatch<'db>(
    env: &Env<'db>,
    expected: Expected,
    span: Span<'db>,
    name_lower: SymTyName<'db>,
    name_upper: SymTyName<'db>,
) -> Reported {
    let (name_expected, name_found) = expected.expected_found(name_lower, name_upper);
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("expected {name_expected}, found {name_found}"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("I expected a {name_expected}, but I found a {name_found}"),
    )
    .report(db)
}

fn report_universal_mismatch<'db>(
    env: &Env<'db>,
    expected: Expected,
    span: Span<'db>,
    univ_lower: SymVariable<'db>,
    univ_upper: SymVariable<'db>,
) -> Reported {
    let db = env.db();
    let (univ_expected, univ_found) = expected.expected_found(univ_lower, univ_upper);

    match (univ_expected.name(db), univ_found.name(db)) {
        (Some(_), _) | (_, Some(_)) => Diagnostic::error(
            db,
            span,
            format!("expected {univ_expected}, found {univ_found}"),
        )
        .label(
            db,
            Level::Error,
            span,
            format!("I expected a {univ_expected}, but I found a {univ_found}"),
        )
        .label(
            db,
            Level::Info,
            univ_expected.span(db),
            format!("{univ_expected} declared here"),
        )
        .label(
            db,
            Level::Info,
            univ_found.span(db),
            format!("{univ_found} declared here"),
        )
        .report(db),

        (None, None) => Diagnostic::error(
            db,
            span,
            format!("expected {univ_expected}, found different {univ_found}"),
        )
        .label(
            db,
            Level::Error,
            span,
            format!("I expected a {univ_expected}, but I found a different {univ_found}"),
        )
        .label(
            db,
            Level::Info,
            univ_expected.span(db),
            format!("first {univ_expected} declared here"),
        )
        .label(
            db,
            Level::Info,
            univ_found.span(db),
            format!("second {univ_found} declared here"),
        )
        .report(db),
    }
}

fn report_numeric_type_expected<'db>(env: &Env<'db>, span: Span<'db>, ty: SymTy<'db>) -> Reported {
    let db = env.db();
    Diagnostic::error(db, span, format!("expected a numeric type, found `{ty}`"))
        .label(
            db,
            Level::Error,
            span,
            format!("I expected a numeric type, but I found a `{ty}`"),
        )
        .report(db)
}

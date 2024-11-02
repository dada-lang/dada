//! Implement object-level subtyping.

use std::future::Future;

use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_ir_sym::{
    indices::InferVarIndex, primitive::SymPrimitiveKind, symbol::SymVariable, ty::SymTyName,
};
use futures::StreamExt;

use crate::{
    bound::Direction,
    env::Env,
    object_ir::{ObjectGenericTerm, ObjectTy, ObjectTyKind},
};

pub async fn require_assignable_object_type<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    value_ty: ObjectTy<'db>,
    place_ty: ObjectTy<'db>,
) -> Errors<()> {
    let db = env.db();

    match (value_ty.kind(db), place_ty.kind(db)) {
        (ObjectTyKind::Never, _) => Ok(()),
        _ => {
            require_sub_object_type(env, Direction::UpperBoundedBy, span, value_ty, place_ty).await
        }
    }
}

pub fn require_sub_object_type<'a, 'db>(
    env: &'a Env<'db>,

    // Tracks which of the types is the "expected" type from a user's point-of-view.
    // This typically starts out as `UpperBoundedBy`, meaning that the `sup` type is
    // the one that is "expected", but it can flip due to variance.
    expected: Direction,

    // Span that is forcing the sub-object comparison. We should probably track a more
    // complex cause.
    span: Span<'db>,

    // Prospective subtype
    lower: ObjectTy<'db>,

    // Prospective supertype
    upper: ObjectTy<'db>,
) -> impl Future<Output = Errors<()>> + use<'a, 'db> {
    Box::pin(async move {
        let db = env.db();

        match (lower.kind(db), upper.kind(db)) {
            (ObjectTyKind::Error(_), _) | (_, ObjectTyKind::Error(_)) => Ok(()),

            (ObjectTyKind::Var(univ_sub), ObjectTyKind::Var(univ_sup)) => {
                if univ_sub == univ_sup {
                    Ok(())
                } else {
                    Err(report_universal_mismatch(env, span, *univ_sub, *univ_sup))
                }
            }

            (&ObjectTyKind::Infer(infer_var), _) => {
                bound_inference_var(
                    env,
                    span,
                    infer_var,
                    expected == Direction::LowerBoundedBy,
                    Direction::UpperBoundedBy,
                    upper.into(),
                )
                .await
            }

            (_, &ObjectTyKind::Infer(infer_var)) => {
                bound_inference_var(
                    env,
                    span,
                    infer_var,
                    expected == Direction::LowerBoundedBy,
                    Direction::LowerBoundedBy,
                    lower.into(),
                )
                .await
            }

            (
                ObjectTyKind::Named(name_lower, args_lower),
                ObjectTyKind::Named(name_upper, args_upper),
            ) => {
                if name_lower != name_upper {
                    return Err(report_class_name_mismatch(
                        env,
                        span,
                        *name_lower,
                        *name_upper,
                    ));
                }

                assert_eq!(args_lower.len(), args_upper.len());

                // FIXME: variance
                for (&arg_lower, &arg_upper) in args_lower.iter().zip(args_upper) {
                    require_sub_object_term(env, expected, span, arg_lower, arg_upper).await?;
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

async fn bound_inference_var<'db>(
    env: &Env<'db>,

    span: Span<'db>,
    infer_var: InferVarIndex,

    // True if the inference variable is the expected type
    infer_var_expected: bool,

    // The relation of `term` to `infer_var`.
    direction: Direction,
    term: ObjectGenericTerm<'db>,
) -> Errors<()> {
    env.runtime()
        .insert_inference_var_bound(infer_var, direction, term);

    let opposite_bounds: Vec<ObjectGenericTerm<'db>> =
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
                        Direction::UpperBoundedBy
                    } else {
                        Direction::LowerBoundedBy
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
                        Direction::LowerBoundedBy
                    } else {
                        Direction::UpperBoundedBy
                    },
                )
            }
        };

        require_sub_object_term(env, expected, span, arg_sub, arg_sup).await?;
    }

    Ok(())
}

async fn require_sub_object_term<'db>(
    env: &Env<'db>,
    expected: Direction,
    span: Span<'db>,
    arg_sub: ObjectGenericTerm<'db>,
    arg_sup: ObjectGenericTerm<'db>,
) -> Errors<()> {
    match (arg_sub, arg_sup) {
        (ObjectGenericTerm::Type(sub), ObjectGenericTerm::Type(sup)) => {
            require_sub_object_type(env, expected, span, sub, sup).await
        }
        (ObjectGenericTerm::Perm, ObjectGenericTerm::Perm)
        | (ObjectGenericTerm::Place, ObjectGenericTerm::Place)
        | (ObjectGenericTerm::Error(_), ObjectGenericTerm::Error(_)) => Ok(()),
        _ => unreachable!("kind mismatch"),
    }
}

pub async fn require_numeric_type<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    start_ty: ObjectTy<'db>,
) -> Errors<()> {
    let db = env.db();

    let mut bounds = env.transitive_upper_bounds(start_ty);
    while let Some(ty) = bounds.next().await {
        match ty.kind(db) {
            ObjectTyKind::Error(_) => {}
            ObjectTyKind::Never => {}
            ObjectTyKind::Named(name, _) => match name {
                SymTyName::Primitive(prim) => match prim.kind(db) {
                    SymPrimitiveKind::Int { .. }
                    | SymPrimitiveKind::Isize
                    | SymPrimitiveKind::Uint { .. }
                    | SymPrimitiveKind::Usize
                    | SymPrimitiveKind::Float { .. } => {}
                    SymPrimitiveKind::Bool | SymPrimitiveKind::Char | SymPrimitiveKind::Str => {
                        return Err(report_numeric_type_expected(env, span, ty))
                    }
                },
                SymTyName::Future | SymTyName::Class(_) | SymTyName::Tuple { .. } => {
                    return Err(report_numeric_type_expected(env, span, ty))
                }
            },
            ObjectTyKind::Var(_) => return Err(report_numeric_type_expected(env, span, ty)),
            ObjectTyKind::Infer(_) => {}
        }
    }

    Ok(())
}

fn report_class_name_mismatch<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    name_sub: SymTyName<'db>,
    name_sup: SymTyName<'db>,
) -> Reported {
    let db = env.db();
    Diagnostic::error(db, span, format!("expected {name_sub}, found {name_sup}"))
        .label(
            db,
            Level::Error,
            span,
            format!("I expected a {name_sup}, but I found a {name_sub}"),
        )
        .report(db)
}

fn report_universal_mismatch<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    univ_sub: SymVariable<'db>,
    univ_sup: SymVariable<'db>,
) -> Reported {
    let db = env.db();

    match (univ_sub.name(db), univ_sup.name(db)) {
        (Some(_), _) | (_, Some(_)) => {
            Diagnostic::error(db, span, format!("expected {univ_sub}, found {univ_sup}"))
                .label(
                    db,
                    Level::Error,
                    span,
                    format!("I expected a {univ_sub}, but I found a {univ_sup}"),
                )
                .label(
                    db,
                    Level::Info,
                    univ_sub.span(db),
                    format!("{univ_sub} declared here"),
                )
                .label(
                    db,
                    Level::Info,
                    univ_sub.span(db),
                    format!("{univ_sup} declared here"),
                )
                .report(db)
        }

        (None, None) => Diagnostic::error(
            db,
            span,
            format!("expected {univ_sub}, found different {univ_sup}"),
        )
        .label(
            db,
            Level::Error,
            span,
            format!("I expected a {univ_sub}, but I found a different {univ_sup}"),
        )
        .label(
            db,
            Level::Info,
            univ_sub.span(db),
            format!("first {univ_sub} declared here"),
        )
        .label(
            db,
            Level::Info,
            univ_sub.span(db),
            format!("second {univ_sup} declared here"),
        )
        .report(db),
    }
}

fn report_numeric_type_expected<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    ty: ObjectTy<'db>,
) -> Reported {
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

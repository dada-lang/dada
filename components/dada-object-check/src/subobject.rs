use std::future::Future;

use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_ir_sym::{primitive::SymPrimitiveKind, symbol::SymVariable, ty::SymTyName};
use futures::StreamExt;

use crate::{
    bound::Bound,
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
        _ => require_sub_object_type(env, span, value_ty, place_ty).await,
    }
}

pub fn require_sub_object_type<'a, 'db>(
    env: &'a Env<'db>,
    span: Span<'db>,
    sub: ObjectTy<'db>,
    sup: ObjectTy<'db>,
) -> impl Future<Output = Errors<()>> + use<'a, 'db> {
    Box::pin(async move {
        let db = env.db();

        match (sub.kind(db), sup.kind(db)) {
            (ObjectTyKind::Error(_), _) | (_, ObjectTyKind::Error(_)) => Ok(()),

            (ObjectTyKind::Var(univ_sub), ObjectTyKind::Var(univ_sup)) => {
                if univ_sub == univ_sup {
                    Ok(())
                } else {
                    Err(report_universal_mismatch(env, span, *univ_sub, *univ_sup))
                }
            }

            (&ObjectTyKind::Infer(var), _) => env.bound_inference_var(var, Bound::UpperBound(sup)),

            (_, &ObjectTyKind::Infer(var)) => env.bound_inference_var(var, Bound::LowerBound(sup)),

            (ObjectTyKind::Named(name_sub, args_sub), ObjectTyKind::Named(name_sup, args_sup)) => {
                if name_sub != name_sup {
                    return Err(report_class_name_mismatch(env, span, *name_sub, *name_sup));
                }

                assert_eq!(args_sub.len(), args_sup.len());

                for (&arg_sub, &arg_sup) in args_sub.iter().zip(args_sup) {
                    require_sub_object_term(env, span, arg_sub, arg_sup).await?;
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

async fn require_sub_object_term<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    arg_sub: ObjectGenericTerm<'db>,
    arg_sup: ObjectGenericTerm<'db>,
) -> Errors<()> {
    match (arg_sub, arg_sup) {
        (ObjectGenericTerm::Type(sub), ObjectGenericTerm::Type(sup)) => {
            require_sub_object_type(env, span, sub, sup).await
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

    let mut bounds = env.bounds(start_ty);
    while let Some(bound) = bounds.next().await {
        let ty = bound.into_term();
        match ty.kind(db) {
            ObjectTyKind::Error(_) => {}
            ObjectTyKind::Never => {}
            ObjectTyKind::Named(name, vec) => match name {
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
            ObjectTyKind::Infer(var) => {}
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

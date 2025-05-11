use crate::{
    check::{CheckTyInEnv, signature::PreparedEnv},
    ir::{
        classes::SymAggregate,
        functions::{SymFunction, SymFunctionSource},
    },
};
use dada_ir_ast::{
    ast::{AstAggregate, AstBlock},
    diagnostic::{Diagnostic, Err, Level},
};
use dada_parser::prelude::FunctionBlock;

use crate::{
    check::runtime::Runtime,
    check::signature::prepare_env,
    ir::exprs::{SymExpr, SymExprKind, SymPlaceExpr, SymPlaceExprKind},
};

use super::{
    CheckExprInEnv, live_places::LivePlaces, report::InvalidReturnValue, resolve::Resolver,
};

pub(crate) fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<SymExpr<'db>> {
    match function.source(db) {
        SymFunctionSource::Function(ast_function) => {
            let block = ast_function.body_block(db)?;
            Some(check_function_body_ast_block(db, function, block))
        }
        SymFunctionSource::Constructor(sym_class, ast_class_item) => Some(
            check_function_body_class_constructor(db, function, sym_class, ast_class_item),
        ),
    }
}

/// Check the automatic construct that results when user writes parentheses, like `class Foo(...)`.
fn check_function_body_class_constructor<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    sym_class: SymAggregate<'db>,
    ast_class_item: AstAggregate<'db>,
) -> SymExpr<'db> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> SymExpr<'db> {
            let PreparedEnv {
                ref mut env,
                input_symbols,
                input_tys,
                ..
            } = prepare_env(db, runtime, function).await;

            let scope = env.scope.clone();
            let self_ty = sym_class.self_ty(db, &scope).check_in_env(env).await;
            let span = ast_class_item.inputs(db).as_ref().unwrap().span;
            let fields = sym_class.fields(db).collect::<Vec<_>>();
            assert_eq!(input_symbols.len(), input_tys.len());

            // Careful: Not allowed to declare other fields.
            let parameter_exprs = input_symbols.iter().zip(&input_tys).map(|(&v, &ty)| {
                SymPlaceExpr::new(db, v.span(db), ty, SymPlaceExprKind::Var(v)).give(db)
            });

            // The first N fields will be the inputs declared in parentheses.
            // But if user declared additional fields, that's an error for now.
            // Eventually perhaps we can support default values.
            let other_exprs = fields[input_symbols.len()..].iter().map(|sym_field| {
                SymExpr::err(
                    db,
                    Diagnostic::error(
                        db,
                        sym_field.name_span(db),
                        "cannot have both explicit fields and an automatic constructor".to_string(),
                    )
                    .label(
                        db,
                        Level::Error,
                        sym_field.name_span(db),
                        "I found an explicit field declaration here".to_string(),
                    )
                    .label(
                        db,
                        Level::Info,
                        span,
                        "I also found an automatic class constructor here",
                    )
                    .report(db),
                )
            });

            SymExpr::new(
                db,
                span,
                self_ty,
                SymExprKind::Aggregate {
                    ty: self_ty,
                    fields: parameter_exprs.chain(other_exprs).collect(),
                },
            )
        },
        |expr| expr,
    )
}

fn check_function_body_ast_block<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    body: AstBlock<'db>,
) -> SymExpr<'db> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| {
            let PreparedEnv {
                mut env,
                output_ty_body,
                ..
            } = prepare_env(db, runtime, function).await;
            env.log("check_function_body_ast_block", &[&function, &body]);
            let live_after = LivePlaces::none(&env);
            let expr = body.check_in_env(&mut env, live_after).await;
            env.spawn_require_assignable_type(
                live_after,
                expr.ty(db),
                output_ty_body,
                &InvalidReturnValue::new(expr, output_ty_body),
            );
            (env, expr)
        },
        |(mut env, expr)| Resolver::new(&mut env).resolve(expr),
    )
}

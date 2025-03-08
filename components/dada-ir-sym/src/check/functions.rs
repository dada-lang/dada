use crate::ir::{
    classes::SymAggregate,
    functions::{SymFunction, SymFunctionSource, SymInputOutput},
    red::RedInfers,
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

use super::CheckInEnv;

pub(crate) fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<(SymExpr<'db>, RedInfers<'db>)> {
    match function.source(db) {
        SymFunctionSource::Function(ast_function) => {
            let Some(block) = ast_function.body_block(db) else {
                return None;
            };
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
) -> (SymExpr<'db>, RedInfers<'db>) {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> SymExpr<'db> {
            let (
                env,
                input_symbols,
                SymInputOutput {
                    input_tys,
                    output_ty: _,
                },
            ) = prepare_env(db, runtime, function).await;

            let scope = env.scope.clone();
            let self_ty = sym_class.self_ty(db, &scope).check_in_env(&env).await;
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
                        format!("cannot have both explicit fields and an automatic constructor"),
                    )
                    .label(
                        db,
                        Level::Error,
                        sym_field.name_span(db),
                        format!("I found an explicit field declaration here"),
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
    )
}

fn check_function_body_ast_block<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    body: AstBlock<'db>,
) -> (SymExpr<'db>, RedInfers<'db>) {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> SymExpr<'db> {
            let (env, _, _) = prepare_env(db, runtime, function).await;
            body.check_in_env(&env).await
        },
    )
}

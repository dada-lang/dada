use dada_ir_ast::{
    ast::{AstAggregate, AstBlock},
    diagnostic::{Diagnostic, Err, Errors, Level},
};
use dada_ir_sym::{
    class::SymAggregate,
    function::{SymFunction, SymFunctionSource, SymInputOutput},
};
use dada_parser::prelude::FunctionBlock;

use crate::{
    check::Runtime,
    env::Env,
    object_ir::{
        ObjectExpr, ObjectExprKind, ObjectFunctionSignature, ObjectInputOutput, ObjectPlaceExpr,
        ObjectPlaceExprKind, ToObjectIr,
    },
    statements::check_block_statements,
    Checking,
};

pub fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<ObjectExpr<'db>> {
    match function.source(db) {
        SymFunctionSource::Function(ast_function) => {
            let Some(block) = ast_function.body_block(db) else {
                return None;
            };
            check_function_body_ast_block(db, function, block)
        }
        SymFunctionSource::Constructor(sym_class, ast_class_item) => Some(
            check_function_body_class_constructor(db, function, sym_class, ast_class_item),
        ),
    }
}

pub fn check_function_signature<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Errors<ObjectFunctionSignature<'db>> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> Errors<ObjectFunctionSignature<'db>> {
            let (
                env,
                SymInputOutput {
                    input_tys,
                    output_ty,
                },
            ) = prepare_env(db, runtime, function);

            let input_output = ObjectInputOutput {
                input_tys: input_tys.to_object_ir(&env),
                output_ty: output_ty.to_object_ir(&env),
            };

            let scope = env.into_scope();
            Ok(ObjectFunctionSignature::new(
                db,
                function.signature(db).symbols(db).clone(),
                scope.into_bound_value(db, input_output),
            ))
        },
    )
}

/// Check the automatic construct that results when user writes parentheses, like `class Foo(...)`.
fn check_function_body_class_constructor<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    sym_class: SymAggregate<'db>,
    ast_class_item: AstAggregate<'db>,
) -> ObjectExpr<'db> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> ObjectExpr<'db> {
            let (
                env,
                SymInputOutput {
                    input_tys,
                    output_ty: _,
                },
            ) = prepare_env(db, runtime, function);

            let scope = env.scope.clone();
            let self_ty = sym_class.self_ty(db, &scope).to_object_ir(&env);
            let span = ast_class_item.inputs(db).as_ref().unwrap().span;
            let input_vars = &function.signature(db).symbols(db).input_variables;
            let fields = sym_class.fields(db).collect::<Vec<_>>();
            assert_eq!(input_vars.len(), input_tys.len());

            // Careful: Not allowed to declare other fields.
            let parameter_exprs = input_vars.iter().zip(&input_tys).map(|(&v, &ty)| {
                let ty = ty.to_object_ir(&env);
                ObjectPlaceExpr::new(db, v.span(db), ty, ObjectPlaceExprKind::Var(v)).give(db)
            });

            // The first N fields will be the inputs declared in parentheses.
            // But if user declared additional fields, that's an error for now.
            // Eventually perhaps we can support default values.
            let other_exprs = fields[input_vars.len()..].iter().map(|sym_field| {
                ObjectExpr::err(
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

            ObjectExpr::new(
                db,
                span,
                self_ty,
                ObjectExprKind::Aggregate {
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
) -> Option<ObjectExpr<'db>> {
    Some(Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> ObjectExpr<'db> {
            let (env, _) = prepare_env(db, runtime, function);

            let expr = body.check(&env).await;

            expr
        },
    ))
}

fn prepare_env<'db>(
    db: &'db dyn crate::Db,
    runtime: &Runtime<'db>,
    function: SymFunction<'db>,
) -> (Env<'db>, SymInputOutput<'db>) {
    let mut env = Env::new(runtime, function.scope(db));

    let signature = function.signature(db);

    // Bring generics + input variables into scope and get the input/output types.
    let SymInputOutput {
        input_tys,
        output_ty,
    } = env.open_universally(runtime, signature.input_output(db));

    // Bring parameters into scope.
    let method_input_variables = &signature.symbols(db).input_variables;
    assert_eq!(input_tys.len(), method_input_variables.len());
    for (&lv, &lv_ty) in method_input_variables.iter().zip(&input_tys) {
        env.set_program_variable_ty(lv, lv_ty);
    }

    // Set return type.
    env.set_return_ty(output_ty);

    (
        env,
        SymInputOutput {
            input_tys,
            output_ty,
        },
    )
}

impl<'db> Checking<'db> for AstBlock<'db> {
    type Checking = ObjectExpr<'db>;

    async fn check(&self, env: &Env<'db>) -> Self::Checking {
        let db = env.db();

        let statements = self.statements(db);
        check_block_statements(env, statements.span, statements).await
    }
}

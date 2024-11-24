use dada_ir_ast::{
    ast::{AstAggregate, AstBlock},
    diagnostic::{Diagnostic, Err, Level},
};
use dada_ir_sym::{
    class::SymAggregate,
    function::{SymFunction, SymFunctionSource, SymInputOutput},
    scope_tree::ScopeTreeNode,
};
use dada_parser::prelude::FunctionBlock;

use crate::{
    check::Runtime,
    env::Env,
    object_ir::{ObjectExpr, ObjectExprKind, ObjectPlaceExpr, ObjectPlaceExprKind},
    prelude::ToObjectIr,
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
        SymFunctionSource::Constructor(sym_class, ast_class_item) => {
            check_function_body_class_constructor(db, function, sym_class, ast_class_item)
        }
    }
}

/// Check the automatic construct that results when user writes parentheses, like `class Foo(...)`.
fn check_function_body_class_constructor<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    sym_class: SymAggregate<'db>,
    ast_class_item: AstAggregate<'db>,
) -> Option<ObjectExpr<'db>> {
    let scope = sym_class.into_scope(db);
    let self_ty = sym_class.self_ty(db, &scope).to_object_ir(db);
    let span = ast_class_item.inputs(db).as_ref().unwrap().span;
    let signature = function.signature(db);
    let input_vars = &signature.input_output(db).bound_value.variables;
    let input_tys = &signature.input_output(db).bound_value.bound_value.input_tys;
    let fields = sym_class.fields(db).collect::<Vec<_>>();
    assert_eq!(input_vars.len(), input_tys.len());

    // Careful: Not allowed to declare other fields.
    let parameter_exprs = input_vars.iter().zip(input_tys).map(|(&v, &ty)| {
        let ty = ty.to_object_ir(db);
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

    Some(ObjectExpr::new(
        db,
        span,
        self_ty,
        ObjectExprKind::Aggregate {
            ty: self_ty,
            fields: parameter_exprs.chain(other_exprs).collect(),
        },
    ))
}

fn check_function_body_ast_block<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    body: AstBlock<'db>,
) -> Option<ObjectExpr<'db>> {
    let scope = function.scope(db);
    Some(Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> ObjectExpr<'db> {
            let mut env = Env::new(runtime, scope);

            let signature = function.signature(db);
            let input_output_binder = signature.input_output(db);

            // Bring class/method generics into scope.
            let method_input_variables = &signature.symbols(db).input_variables;
            let SymInputOutput {
                input_tys,
                output_ty,
            } = env.open_universally(runtime, method_input_variables, input_output_binder);

            // Bring parameters into scope.
            assert_eq!(input_tys.len(), method_input_variables.len());
            for (&lv, &lv_ty) in method_input_variables.iter().zip(&input_tys) {
                env.insert_program_variable(lv, lv_ty);
            }

            // Set return type.
            env.set_return_ty(output_ty);

            let expr = body.check(&env).await;

            expr
        },
    ))
}

impl<'db> Checking<'db> for AstBlock<'db> {
    type Checking = ObjectExpr<'db>;

    async fn check(&self, env: &Env<'db>) -> Self::Checking {
        let db = env.db();

        let statements = self.statements(db);
        check_block_statements(env, statements.span, statements).await
    }
}

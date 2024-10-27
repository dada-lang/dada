use dada_ir_ast::ast::AstBlock;
use dada_ir_sym::function::{SymFunction, SymInputOutput};

use crate::{
    check::Check, env::Env, object_ir::ObjectExpr, statements::check_block_statements, Checking,
};

pub fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<ObjectExpr<'db>> {
    let Some(body) = function.ast_body(db) else {
        return None;
    };

    let scope = function.scope(db);
    Some(Check::execute(
        db,
        function.name_span(db),
        async move |check| -> ObjectExpr<'db> {
            let mut env = Env::new(scope);

            let signature = function.signature(db);
            let input_output_binder = signature.input_output(db);

            // Bring class/method generics into scope.
            let method_input_variables = &signature.symbols(db).input_variables;
            let SymInputOutput {
                input_tys,
                output_ty,
            } = env.open_universally(check, method_input_variables, input_output_binder);

            // Bring parameters into scope.
            assert_eq!(input_tys.len(), method_input_variables.len());
            for (&lv, &lv_ty) in method_input_variables.iter().zip(&input_tys) {
                env.insert_program_variable(lv, lv_ty);
            }

            // Set return type.
            env.set_return_ty(output_ty);

            let expr = body.check(&check, &env).await;

            expr
        },
    ))
}

impl<'db> Checking<'db> for AstBlock<'db> {
    type Checking = ObjectExpr<'db>;

    async fn check(&self, check: &Check<'db>, env: &Env<'db>) -> Self::Checking {
        let db = check.db;

        let statements = self.statements(db);
        check_block_statements(check, env, statements.span, statements).await
    }
}

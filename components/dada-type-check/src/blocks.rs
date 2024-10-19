use dada_ir_ast::ast::AstBlock;
use dada_ir_sym::{
    function::{SymFunction, SymInputOutput},
    ty::SymTy,
};

use crate::{
    check::Check,
    env::Env,
    ir::CheckedExpr,
    object_ir::{ObjectExpr, ObjectExprKind, ObjectTy},
    Checking,
};

pub fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<CheckedExpr<'db>> {
    let Some(body) = function.ast_body(db) else {
        return None;
    };

    let scope = function.scope(db);
    Some(Check::execute(
        db,
        function.name_span(db),
        &arenas,
        async |check| {
            let mut env = Env::new(scope);

            let symbols = signature.symbols(db);

            let class_symbols = &symbols.generics[0..];

            // Bring class/method generics into scope.
            let signature = function.signature(db);
            let SymInputOutput {
                input_tys,
                output_ty,
            } = env.open_universally2(
                &check,
                &signature.symbols(db).generics,
                signature.input_output(db),
            );

            // Bring parameters into scope.
            assert_eq!(input_tys.len(), signature.symbols(db).inputs.len());
            for (&lv, &lv_ty) in signature.symbols(db).inputs.iter().zip(&input_tys) {
                env.insert_program_variable(lv, lv_ty);
            }

            // Set return type.
            env.set_return_ty(output_ty);

            body.check(&check, &env).await
        },
    ))
}

impl<'db> Checking<'db> for AstBlock<'db> {
    type Checking = ObjectExpr<'db>;

    async fn check(&self, check: &Check<'db>, env: &Env<'db>) -> Self::Checking {
        let db = check.db;

        let statements = self.statements(db);

        if statements.is_empty() {
            return ObjectExpr::new(
                db,
                statements.span,
                ObjectTy::unit(db),
                ObjectExprKind::Tuple(vec![]),
            );
        }

        statements.values[..].check(check, env).await
    }
}

use dada_ir_ast::ast::{AstBlock, AstStatement};
use dada_ir_sym::{
    function::{SymFunction, SymInputOutput},
    prelude::IntoSymInScope,
    ty::SymTy,
};

use crate::{
    checking_ir::{Expr, ExprKind},
    env::Env,
    executor::{Check, ExecutorArenas},
    ir::CheckedExpr,
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
    let arenas = ExecutorArenas::default();
    let check = &mut Check::new(db, &arenas);
    let mut env = Env::new(scope);

    // Bring class/method generics into scope.
    let signature = function.signature(db);
    let SymInputOutput {
        input_tys,
        output_ty,
    } = env.open_universally2(
        check,
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

    let checking_expr = body.check(check, env);
    todo!()
}

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for AstBlock<'db> {
    type Checking = Expr<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, env: Env<'db>) -> Self::Checking {
        let db = check.db;

        let statements = self.statements(db);

        if statements.is_empty() {
            return check.expr(statements.span, SymTy::unit(db), ExprKind::Tuple(vec![]));
        }

        statements.values[..].check(check, env)
    }
}

use dada_ir_ast::ast::{AstBlock, AstStatement};
use dada_ir_sym::{function::SymFunction, ty::SymTy};

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
    let mut check = Check::new(db, scope, &arenas);
    let env = &Env::new();
    let checking_expr = body.check(&mut check, env);
    todo!()
}

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for AstBlock<'db> {
    type Checking = Expr<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, env: &Env<'db>) -> Self::Checking {
        let db = check.db;

        let statements = self.statements(db);

        if statements.is_empty() {
            return check.expr(statements.span, SymTy::unit(db), ExprKind::Tuple(vec![]));
        }

        statements.values[..].check(check, env)
    }
}

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for [AstStatement<'db>] {
    type Checking = Expr<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, env: &Env<'db>) -> Self::Checking {
        let Some((first, rest)) = self.split_first() else {
            panic!("empty list of statements")
        };

        match first {
            AstStatement::Let(s) => {
                todo!()
            }
            AstStatement::Expr(e) => {
                let ce = e.check(check, env);
                if rest.is_empty() {
                    ce
                } else {
                    let re = rest.check(check, env);
                    check.expr(ce.span.to(re.span), re.ty, ExprKind::Semi(ce, re))
                }
            }
        }
    }
}

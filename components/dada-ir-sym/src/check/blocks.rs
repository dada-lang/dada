use dada_ir_ast::ast::AstBlock;

use crate::{check::env::Env, check::statements::check_block_statements, ir::exprs::SymExpr};

use super::CheckInEnv;

impl<'db> CheckInEnv<'db> for AstBlock<'db> {
    type Output = SymExpr<'db>;

    async fn check_in_env(&self, env: &Env<'db>) -> Self::Output {
        let db = env.db();

        let statements = self.statements(db);
        check_block_statements(env, statements.span, statements).await
    }
}

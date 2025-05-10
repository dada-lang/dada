use dada_ir_ast::ast::AstBlock;

use crate::{check::env::Env, check::statements::check_block_statements, ir::exprs::SymExpr};

use super::{CheckExprInEnv, live_places::LivePlaces};

impl<'db> CheckExprInEnv<'db> for AstBlock<'db> {
    type Output = SymExpr<'db>;

    async fn check_in_env(&self, env: &mut Env<'db>, live_after: LivePlaces) -> Self::Output {
        let db = env.db();

        let statements = self.statements(db);
        check_block_statements(env, live_after, statements.span, statements).await
    }
}

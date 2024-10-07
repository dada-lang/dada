use dada_ir_ast::ast::AstBlock;
use dada_ir_sym::function::SymFunction;

use crate::{env::Env, ir::CheckedBlock, CheckInEnv};

pub fn check_function_body<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Option<CheckedBlock<'db>> {
    if let Some(body) = function.ast_body(db) {
        let scope = function.scope(db).with_body_link();
        let env = &mut Env::new(db, &scope);
        Some(body.check_in_env(db, env))
    } else {
        None
    }
}

impl<'db> CheckInEnv<'db> for AstBlock<'db> {
    type Checked = CheckedBlock<'db>;
    fn check_in_env(&self, db: &'db dyn crate::Db, env: &mut Env<'_, 'db>) -> Self::Checked {
        let statements = self
            .statements(db)
            .iter()
            .map(|s| s.check_in_env(db, env))
            .collect();
        CheckedBlock::new(db, statements)
    }
}

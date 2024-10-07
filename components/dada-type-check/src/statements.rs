use dada_ir_ast::ast::AstStatement;

use crate::{env::Env, ir::CheckedStatement, CheckInEnv};

impl<'db> CheckInEnv<'db> for AstStatement<'db> {
    type Checked = CheckedStatement<'db>;

    fn check_in_env(&self, _db: &'db dyn crate::Db, _env: &mut Env<'_, 'db>) -> Self::Checked {
        todo!()
    }
}

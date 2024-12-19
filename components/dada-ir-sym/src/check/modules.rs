use crate::{ir::module::SymModule, prelude::CheckUseItems};
use dada_ir_ast::{diagnostic::Errors, span::Spanned};

use super::{scope::Resolve, Env, Runtime};

/// Resolve all use items found in this module.
/// This is executed by `dada-ir-check` crate
/// simply to force errors to be reported.
impl<'db> CheckUseItems<'db> for SymModule<'db> {
    fn check_use_items(self, db: &'db dyn crate::Db) {
        let _: Errors<()> = Runtime::execute(db, self.span(db), async move |runtime| {
            let env = Env::new(runtime, self.mod_scope(db));
            for item in self.ast_use_map(db).values() {
                let _ = item.path(db).resolve_in(&env);
            }
            Ok(())
        });
    }
}

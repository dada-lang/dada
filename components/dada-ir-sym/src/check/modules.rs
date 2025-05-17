use crate::{ir::module::SymModule, prelude::CheckUseItems};
use dada_ir_ast::{diagnostic::Errors, span::Spanned};

use super::{Env, Runtime, scope::Resolve};

/// Resolve all use items found in this module.
/// This is executed by `dada-ir-check` crate
/// simply to force errors to be reported.
#[salsa::tracked]
impl<'db> CheckUseItems<'db> for SymModule<'db> {
    #[salsa::tracked]
    fn check_use_items(self, db: &'db dyn crate::Db) {
        let _: Errors<()> = Runtime::execute(
            db,
            self.span(db),
            "check_use_items",
            &[&self],
            async move |runtime| {
                let mut env = Env::new(runtime, self.mod_scope(db));
                for item in self.ast_use_map(db).values() {
                    let _ = item.path(db).resolve_in(&mut env).await;
                }
                Ok(())
            },
            |v| v,
        );
    }
}

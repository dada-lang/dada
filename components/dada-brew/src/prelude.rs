use dada_ir::{code::bir, function::Function, item::Item};
use dada_validate::prelude::*;

#[extension_trait::extension_trait]
pub impl FunctionBrewExt for Function {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir {
        let tree = self.validated_tree(db);
        crate::brew::brew(db, tree)
    }
}

#[extension_trait::extension_trait]
pub impl ItemBrewExt for Item {
    fn maybe_brew(self, db: &dyn crate::Db) -> Option<bir::Bir> {
        self.validated_tree(db)
            .map(|tree| crate::brew::brew(db, tree))
    }
}

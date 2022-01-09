use dada_ir::{code::bir, function::Function, item::Item};
use dada_validate::prelude::*;

pub trait BrewExt {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir;
}

impl BrewExt for Function {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir {
        let tree = self.validated_tree(db);
        crate::brew::brew(db, tree)
    }
}

pub trait MaybeBrewExt {
    fn maybe_brew(self, db: &dyn crate::Db) -> Option<bir::Bir>;
}

impl MaybeBrewExt for Item {
    fn maybe_brew(self, db: &dyn crate::Db) -> Option<bir::Bir> {
        self.validated_tree(db)
            .map(|tree| crate::brew::brew(db, tree))
    }
}

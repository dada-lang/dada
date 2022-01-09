use dada_ir::{
    code::{bir, Code},
    function::Function,
    item::Item,
};
use dada_validate::prelude::*;

pub trait BrewExt {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir;
}

impl BrewExt for Code {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir {
        let validated = self.validated_tree(db);
        crate::brew::brew(db, validated)
    }
}

impl BrewExt for Function {
    fn brew(self, db: &dyn crate::Db) -> bir::Bir {
        self.code(db).brew(db)
    }
}

pub trait MaybeBrewExt {
    fn maybe_brew(self, db: &dyn crate::Db) -> Option<bir::Bir>;
}

impl MaybeBrewExt for Item {
    fn maybe_brew(self, db: &dyn crate::Db) -> Option<bir::Bir> {
        self.code(db).map(|code| code.brew(db))
    }
}

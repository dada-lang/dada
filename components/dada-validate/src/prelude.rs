use dada_ir::{
    code::{validated, Code},
    filename::Filename,
    func::Function,
    item::Item,
};

pub trait DadaValidateFilenameExt {
    /// Validates that the root definitions of the file are ok
    /// (shallowly).
    fn validate_root(self, db: &dyn crate::Db);
}

impl DadaValidateFilenameExt for Filename {
    fn validate_root(self, db: &dyn crate::Db) {
        crate::validate::root_definitions(db, self);
    }
}

pub trait DadaValidateCodeExt {
    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree;
}

impl DadaValidateCodeExt for Code {
    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree {
        crate::validate::validate_code(db, self)
    }
}

pub trait DadaValidateFunctionExt {
    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree;
}

impl DadaValidateFunctionExt for Function {
    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree {
        self.code(db).validated_tree(db)
    }
}

pub trait DadaValidateItemExt {
    fn validated_tree(self, db: &dyn crate::Db) -> Option<validated::Tree>;
}

impl DadaValidateItemExt for Item {
    fn validated_tree(self, db: &dyn crate::Db) -> Option<validated::Tree> {
        Some(self.code(db)?.validated_tree(db))
    }
}

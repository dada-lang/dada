use dada_ir::code::{validated, Code};

pub trait CodeExt {
    fn validated_ast(self, db: &dyn crate::Db) -> validated::Tree;
}

impl CodeExt for Code {
    fn validated_ast(self, db: &dyn crate::Db) -> validated::Tree {
        crate::validate::validate_code(db, self)
    }
}

use dada_ir::{class::Class, word::Word};
use dada_validate::prelude::*;

#[extension_trait::extension_trait]
pub impl DadaExecuteClassExt for Class {
    /// All fields of the given class (in order)
    fn field_names(self, db: &dyn crate::Db) -> &Vec<Word> {
        class_field_names(db, self)
    }

    /// Returns the index of the field named `name`, or `None` if there isn't one
    fn field_index(self, db: &dyn crate::Db, name: Word) -> Option<usize> {
        self.field_names(db).iter().position(|w| *w == name)
    }
}

#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn class_field_names(db: &dyn crate::Db, class: Class) -> Vec<Word> {
    class.signature(db).inputs.iter().map(|p| p.name).collect()
}

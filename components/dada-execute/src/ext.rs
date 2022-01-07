use dada_ir::{class::Class, word::Word};
use dada_parse::prelude::*;

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

#[salsa::memoized(in crate::Jar ref)]
pub fn class_field_names(db: &dyn crate::Db, class: Class) -> Vec<Word> {
    class.fields(db).iter().map(|p| p.name(db)).collect()
}

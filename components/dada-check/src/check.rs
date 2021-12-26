use dada_ir::{filename::Filename, item::Item};
use dada_parse::prelude::*;
use dada_validate::prelude::*;

#[salsa::memoized(in crate::Jar)]
pub fn check_filename(db: &dyn crate::Db, filename: Filename) {
    let items = filename.items(db);

    filename.validate_root(db);

    for &item in items {
        match item {
            Item::Function(function) => {
                function.parameters(db);
                function.syntax_tree(db);
                function.validated_tree(db);
            }
            Item::Class(class) => {
                class.fields(db);
            }
        }
    }
}

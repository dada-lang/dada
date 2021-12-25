use dada_ir::{filename::Filename, item::Item};
use dada_parse::prelude::*;
use dada_validate::prelude::CodeExt;

#[salsa::memoized(in crate::Jar)]
pub fn check_filename(db: &dyn crate::Db, filename: Filename) {
    let items = filename.items(db);

    for &item in items {
        match item {
            Item::Function(function) => {
                function.parameters(db);
                function.syntax_tree(db);
                function.code(db).validated_ast(db);
            }
            Item::Class(class) => {
                class.fields(db);
            }
        }
    }
}

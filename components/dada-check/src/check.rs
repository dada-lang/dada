use dada_ir::item::Item;
use dada_ir::word::Word;
use dada_parse::prelude::*;

#[salsa::memoized(in crate::Jar)]
pub fn check_filename(db: &dyn crate::Db, filename: Word) {
    let items = dada_parse::parse_file(db, filename);

    for &item in items {
        match item {
            Item::Function(function) => {
                function.parameters(db);
                function.syntax_tree(db);
            }
            Item::Class(class) => {
                class.fields(db);
            }
        }
    }
}

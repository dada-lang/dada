use dada_ir::{input_file::InputFile, item::Item};
use dada_parse::prelude::*;
use dada_validate::prelude::*;

#[salsa::tracked]
pub fn check_input_file(db: &dyn crate::Db, input_file: InputFile) {
    let items = input_file.items(db);

    input_file.validate_root(db);

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

use dada_ir::{
    code::syntax, function::Function, input_file::InputFile, item::Item, source_file::SourceFile,
};

#[extension_trait::extension_trait]
pub impl DadaParseItemExt for Item {
    fn syntax_tree(self, db: &dyn crate::Db) -> Option<syntax::Tree> {
        match self {
            Item::Function(f) => Some(f.syntax_tree(db)),
            Item::Class(_) => None,
        }
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseFunctionExt for Function {
    /// Returns the Ast for a function.
    fn syntax_tree(self, db: &dyn crate::Db) -> syntax::Tree {
        crate::code_parser::parse_function_body(db, self)
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseInputFileExt for InputFile {
    fn source_file(self, db: &dyn crate::Db) -> &SourceFile {
        crate::file_parser::parse_file(db, self)
    }

    fn items(self, db: &dyn crate::Db) -> &Vec<Item> {
        self.source_file(db).items(db)
    }
}

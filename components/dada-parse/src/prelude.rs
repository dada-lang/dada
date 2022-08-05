use dada_ir::{
    class::Class, code::syntax, function::Function, input_file::InputFile, item::Item,
    parameter::Parameter, source_file::SourceFile,
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

    fn parameters(self, db: &dyn crate::Db) -> &[Parameter] {
        crate::parameter_parser::parse_function_parameters(db, self)
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseClassExt for Class {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parameter_parser::parse_class_parameters(db, self)
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

use dada_ir::{
    class::Class,
    code::{syntax, Code},
    filename::Filename,
    function::Function,
    item::Item,
    parameter::Parameter,
    source_file::SourceFile,
};

#[extension_trait::extension_trait]
pub impl DadaParseItemExt for Item {
    fn syntax_tree(self, db: &dyn crate::Db) -> Option<syntax::Tree> {
        Some(self.code(db)?.syntax_tree(db))
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseCodeExt for Code {
    fn parameters(self, db: &dyn crate::Db) -> &[Parameter] {
        crate::parameter_parser::parse_parameters(db, self.parameter_tokens)
    }

    fn syntax_tree(self, db: &dyn crate::Db) -> syntax::Tree {
        crate::code_parser::parse_code(db, self)
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseFunctionExt for Function {
    /// Returns the Ast for a function.
    fn syntax_tree(self, db: &dyn crate::Db) -> syntax::Tree {
        self.code(db).syntax_tree(db)
    }

    fn parameters(self, db: &dyn crate::Db) -> &[Parameter] {
        self.code(db).parameters(db)
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseClassExt for Class {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parameter_parser::parse_parameters(db, self.field_tokens(db))
    }
}

#[extension_trait::extension_trait]
pub impl DadaParseFilenameExt for Filename {
    fn source_file(self, db: &dyn crate::Db) -> &SourceFile {
        crate::file_parser::parse_file(db, self)
    }

    fn items(self, db: &dyn crate::Db) -> &Vec<Item> {
        self.source_file(db).items(db)
    }
}

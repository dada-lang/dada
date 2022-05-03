use dada_ir::{
    class::Class,
    code::{bir, syntax, Code},
    filename::Filename,
    function::Function,
    item::Item,
    parameter::Parameter,
    source_file::SourceFile,
    span::FileSpan,
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
        if let Some(parameter_tokens) = self.parameter_tokens {
            crate::parameter_parser::parse_parameters(db, parameter_tokens)
        } else {
            &[]
        }
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
pub impl DadaBirSpanExt for bir::Bir {
    /// Given a `syntax_expr` within this BIR, find its span. This operation
    /// is to be avoided unless reporting a diagnostic or really needed, because
    /// it induces a dependency on the *precise span* of the expression and hence
    /// will require re-execution if most anything in the source file changes, even
    /// just adding whitespace.
    fn span_of(self, db: &dyn crate::Db, syntax_expr: syntax::Expr) -> FileSpan {
        let function = self.origin(db);
        let filename = function.filename(db);
        let syntax_tree = function.syntax_tree(db);
        syntax_tree.spans(db)[syntax_expr].in_file(filename)
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

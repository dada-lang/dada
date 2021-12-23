use dada_ir::{
    class::Class,
    code::{syntax, Code},
    filename::Filename,
    func::Function,
    item::Item,
    parameter::Parameter,
};

pub trait CodeExt {
    /// Returns the Ast for a function.
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree;
    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans;
}

impl CodeExt for Code {
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree {
        crate::code_parser::parse_code(db, self)
    }

    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans {
        crate::code_parser::spans_for_parsed_code(db, self)
    }
}

pub trait FunctionExt {
    /// Returns the Ast for a function.
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree;
    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans;
    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter>;
}

impl FunctionExt for Function {
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree {
        self.code(db).syntax_tree(db)
    }

    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans {
        self.code(db).syntax_tree_spans(db)
    }

    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parameter_parser::parse_parameters(db, self.unparsed_parameters(db))
    }
}

pub trait ClassExt {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter>;
}

impl ClassExt for Class {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parameter_parser::parse_parameters(db, self.unparsed_parameters(db))
    }
}

pub trait FilenameExt {
    fn items(self, db: &dyn crate::Db) -> &Vec<Item>;
}

impl FilenameExt for Filename {
    fn items(self, db: &dyn crate::Db) -> &Vec<Item> {
        crate::file_parser::parse_file(db, self)
    }
}

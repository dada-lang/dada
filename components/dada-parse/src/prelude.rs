use dada_ir::{class::Class, code::syntax, func::Function, parameter::Parameter};

pub trait FunctionExt {
    /// Returns the Ast for a function.
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree;
    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans;
    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter>;
}

impl FunctionExt for Function {
    fn syntax_tree(self, db: &dyn crate::Db) -> &syntax::Tree {
        crate::parse_code(db, self.code(db))
    }

    fn syntax_tree_spans(self, db: &dyn crate::Db) -> &syntax::Spans {
        crate::spans_for_parsed_code(db, self.code(db))
    }

    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parse_parameters(db, self.unparsed_parameters(db))
    }
}

pub trait ClassExt {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter>;
}

impl ClassExt for Class {
    fn fields(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::parse_parameters(db, self.unparsed_parameters(db))
    }
}

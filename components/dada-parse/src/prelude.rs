use dada_ir::{
    class::Class,
    code::{Ast, Spans},
    func::Function,
    parameter::Parameter,
};

pub trait FunctionExt {
    /// Returns the Ast for a function.
    fn ast(self, db: &dyn crate::Db) -> &Ast;
    fn spans(self, db: &dyn crate::Db) -> &Spans;
    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter>;
}

impl FunctionExt for Function {
    fn ast(self, db: &dyn crate::Db) -> &Ast {
        crate::parse_code(db, self.code(db))
    }

    fn spans(self, db: &dyn crate::Db) -> &Spans {
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

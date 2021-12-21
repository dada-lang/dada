use dada_ir::{
    code::{Ast, Spans},
    func::Function,
};

pub trait FunctionExt {
    /// Returns the Ast for a function.
    fn ast(self, db: &dyn crate::Db) -> &Ast;
    fn spans(self, db: &dyn crate::Db) -> &Spans;
}

impl FunctionExt for Function {
    fn ast(self, db: &dyn crate::Db) -> &Ast {
        crate::parse_code(db, self.code(db))
    }

    fn spans(self, db: &dyn crate::Db) -> &Spans {
        crate::spans_for_parsed_code(db, self.code(db))
    }
}

use dada_ir::{code::Ast, func::Function};

pub trait FunctionExt {
    fn ast(self, db: &dyn crate::Db) -> &Ast;
}

impl FunctionExt for Function {
    fn ast(self, db: &dyn crate::Db) -> &Ast {
        crate::parse_code(db, self.code(db))
    }
}

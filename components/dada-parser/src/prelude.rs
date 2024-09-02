use dada_ir_ast::ast::Member;

use super::*;

pub trait SourceFileParse {
    fn parse(self, db: &dyn crate::Db) -> Module<'_>;
}

pub trait ClassItemMembers<'db> {
    fn members(self, db: &'db dyn crate::Db) -> AstVec<'db, Member<'db>>;
}

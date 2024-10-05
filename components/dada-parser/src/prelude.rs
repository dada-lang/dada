use dada_ir_ast::ast::{AstBlock, AstMember};

use super::*;

/// Given a [`SourceFile`], parse its members
pub trait SourceFileParse {
    fn parse(self, db: &dyn crate::Db) -> AstModule<'_>;
}

/// Given a [`ClassItem`], parse its members
pub trait ClassItemMembers<'db> {
    fn members(self, db: &'db dyn crate::Db) -> SpanVec<'db, AstMember<'db>>;
}

/// Given a [`Function`], parse its associated body into a block
pub trait FunctionBlock<'db> {
    fn body_block(self, db: &'db dyn crate::Db) -> Option<AstBlock<'db>>;
}

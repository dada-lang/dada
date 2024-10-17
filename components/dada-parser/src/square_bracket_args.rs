use dada_ir_ast::ast::{AstGenericTerm, AstTy, SpanVec, SquareBracketArgs};

use crate::Parser;

#[salsa::tracked]
impl<'db> crate::prelude::SquareBracketArgs<'db> for SquareBracketArgs<'db> {
    #[salsa::tracked]
    fn parse_as_generics(
        self,
        db: &'db dyn crate::Db,
    ) -> SpanVec<'db, dada_ir_ast::ast::AstGenericTerm<'db>> {
        let deferred = self.deferred(db);
        let anchor = deferred.span.anchor;
        Parser::deferred(db, anchor, deferred, |parser| {
            parser.parse_many_and_report_diagnostics::<AstGenericTerm<'db>>(db)
        })
    }
}

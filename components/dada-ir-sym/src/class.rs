use dada_ir_ast::{
    ast::{AstClassItem, AstModule},
    span::Spanned,
};

#[salsa::tracked]
pub struct SymClass<'db> {
    module: AstModule<'db>,
    source: AstClassItem<'db>,
}

impl<'db> Spanned<'db> for SymClass<'db> {
    fn span(&self, db: &'db dyn salsa::Database) -> dada_ir_ast::span::Span<'db> {
        self.source(db).name_span(db)
    }
}

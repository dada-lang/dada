use dada_ir_ast::{
    ast::{AstClassItem, AstModule, Identifier},
    span::{Span, Spanned},
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

impl<'db> SymClass<'db> {
    pub fn name(&self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db)
    }

    /// Number of generic parameters
    pub fn len_generics(&self, db: &'db dyn crate::Db) -> usize {
        if let Some(generics) = self.source(db).generics(db) {
            generics.len()
        } else {
            0
        }
    }

    /// Span of the class name, typically used in diagnostics
    pub fn name_span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.source(db).name_span(db)
    }

    /// Span where generics are declared (possibly the name span, if there are no generics)
    pub fn generics_span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        if let Some(generics) = self.source(db).generics(db) {
            generics.span
        } else {
            self.name_span(db)
        }
    }
}

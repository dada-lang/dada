use crate::span::{Span, Spanned};

use super::Identifier;

/// `class $name { ... }`
#[salsa::tracked]
pub struct AstClassItem<'db> {
    pub span: Span<'db>,

    #[id]
    pub name: Identifier<'db>,

    pub name_span: Span<'db>,

    #[return_ref]
    pub contents: String,
}

impl<'db> AstClassItem<'db> {}

impl<'db> Spanned<'db> for AstClassItem<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstClassItem::span(*self, db)
    }
}

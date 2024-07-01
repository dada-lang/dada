use crate::span::{Span, Spanned};

use super::Identifier;

/// `class $name { ... }`
#[salsa::tracked]
pub struct ClassItem<'db> {
    pub span: Span<'db>,

    #[id]
    pub name: Identifier<'db>,

    pub name_span: Span<'db>,

    #[return_ref]
    pub contents: String,
}

impl<'db> ClassItem<'db> {}

impl<'db> Spanned<'db> for ClassItem<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        ClassItem::span(*self, db)
    }
}

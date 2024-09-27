use crate::span::{Span, Spanned};

use super::{AstPath, SpannedIdentifier};

/// `use $crate.$path [as $id]`
#[salsa::tracked]
pub struct AstUseItem<'db> {
    pub span: Span<'db>,
    pub crate_name: SpannedIdentifier<'db>,
    #[return_ref]
    pub path: AstPath<'db>,
    pub as_id: Option<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for AstUseItem<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstUseItem::span(*self, db)
    }
}

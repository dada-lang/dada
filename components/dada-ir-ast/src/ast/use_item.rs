use crate::span::{Span, Spanned};

use super::{Path, SpannedIdentifier};

/// `use $path [as $id]`
#[salsa::tracked]
pub struct UseItem<'db> {
    pub span: Span<'db>,
    pub path: Path<'db>,
    pub id: Option<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for UseItem<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        UseItem::span(*self, db)
    }
}

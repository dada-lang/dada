use dada_util::SalsaSerialize;

use crate::span::{Span, Spanned};

use super::{AstPath, SpannedIdentifier};

/// `use $crate.$path [as $id]`
#[derive(SalsaSerialize)]
#[salsa::tracked]
pub struct AstUse<'db> {
    pub span: Span<'db>,
    pub crate_name: SpannedIdentifier<'db>,
    #[return_ref]
    pub path: AstPath<'db>,
    pub as_id: Option<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for AstUse<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstUse::span(*self, db)
    }
}

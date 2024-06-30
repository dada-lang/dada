use crate::span::Span;

use super::{Path, SpannedIdentifier};

/// `use $path [as $id]`
#[salsa::tracked]
pub struct UseItem<'db> {
    pub span: Span<'db>,
    pub path: Path<'db>,
    pub id: Option<SpannedIdentifier<'db>>,
}

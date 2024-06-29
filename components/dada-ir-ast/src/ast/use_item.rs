use crate::span::Span;

use super::{Identifier, Path};

/// `use $path [as $id]`
#[salsa::tracked]
pub struct UseItem<'db> {
    pub span: Span<'db>,
    pub path: Path<'db>,
    pub id: Option<Identifier<'db>>,
}

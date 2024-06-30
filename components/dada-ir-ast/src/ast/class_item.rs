use crate::span::Span;

use super::Identifier;

/// `class $name { ... }`
#[salsa::tracked]
pub struct ClassItem<'db> {
    pub span: Span<'db>,

    #[id]
    pub name: Identifier<'db>,

    pub name_span: Span<'db>,

    contents: String,
}

impl<'db> ClassItem<'db> {}

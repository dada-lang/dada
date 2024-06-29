use salsa::{DebugWithDb, Update};

use crate::span::Span;

use super::Identifier;

/// `class $name { ... }`
#[salsa::tracked]
pub struct ClassItem<'db> {
    pub span: Span<'db>,

    pub name: Identifier<'db>,

    contents: String,
}

impl<'db> ClassItem<'db> {}

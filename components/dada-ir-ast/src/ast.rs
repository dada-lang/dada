#![allow(clippy::unused_unit)] // FIXME: derive(Update) triggers these

use salsa::Update;

use crate::{
    inputs::SourceFile,
    span::{Span, Spanned},
};

mod use_item;
pub use use_item::*;
mod class_item;
pub use class_item::*;
mod member;
pub use member::*;
mod function;
pub use function::*;
mod types;
pub use types::*;
mod util;
pub use util::*;
mod expr;
pub use expr::*;

#[salsa::interned]
pub struct Identifier<'db> {
    #[return_ref]
    pub text: String,
}

impl<'db> std::fmt::Display for Identifier<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| write!(f, "{}", self.text(db)))
            .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[salsa::tracked]
pub struct AstModule<'db> {
    #[return_ref]
    pub items: SpanVec<'db, AstItem<'db>>,
}

impl<'db> Spanned<'db> for AstModule<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.items(db).span
    }
}

add_from_impls! {
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update)]
pub enum AstItem<'db> {
    SourceFile(SourceFile),
    Use(UseItem<'db>),
    Class(AstClassItem<'db>),
    Function(AstFunction<'db>),
}
}

/// Path of identifiers (must be non-empty)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct Path<'db> {
    pub ids: Vec<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for Path<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        let len = self.ids.len();
        assert!(len > 0);
        self.ids[0].span.to(self.ids[len - 1].span)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SpannedIdentifier<'db> {
    pub span: Span<'db>,
    pub id: Identifier<'db>,
}

impl<'db> Spanned<'db> for SpannedIdentifier<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

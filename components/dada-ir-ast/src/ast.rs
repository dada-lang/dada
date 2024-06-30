#![allow(clippy::unused_unit)] // FIXME: derive(Update) triggers these

use salsa::{DebugWithDb, Update};

use crate::{
    inputs::SourceFile,
    span::{AbsoluteOffset, Offset, Span},
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
    text: String,
}

#[salsa::tracked]
pub struct Module<'db> {
    items: Vec<Item<'db>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DebugWithDb, Debug, Update)]
pub enum Item<'db> {
    SourceFile(SourceFile),
    Use(UseItem<'db>),
    Class(ClassItem<'db>),
}

impl<'db> Item<'db> {
    pub fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            Item::SourceFile(source_file) => Span {
                anchor: *self,
                start: Offset::ZERO,
                end: Offset::from(source_file.contents(db).len()),
            },
            Item::Use(data) => data.span(db),
            Item::Class(data) => data.span(db),
        }
    }

    pub fn absolute_start(&self, db: &'db dyn crate::Db) -> (SourceFile, AbsoluteOffset) {
        match self {
            Item::SourceFile(source_file) => (*source_file, AbsoluteOffset::ZERO),
            Item::Use(data) => data.span(db).absolute_start(db),
            Item::Class(data) => data.span(db).absolute_start(db),
        }
    }
}

impl<'db> From<UseItem<'db>> for Item<'db> {
    fn from(value: UseItem<'db>) -> Self {
        Item::Use(value)
    }
}

impl<'db> From<ClassItem<'db>> for Item<'db> {
    fn from(value: ClassItem<'db>) -> Self {
        Item::Class(value)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct Path<'db> {
    pub ids: Vec<SpannedIdentifier<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct SpannedIdentifier<'db> {
    pub span: Span<'db>,
    pub id: Identifier<'db>,
}

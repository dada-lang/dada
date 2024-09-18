#![allow(clippy::unused_unit)] // FIXME: derive(Update) triggers these

use salsa::Update;

use crate::{
    inputs::SourceFile,
    span::{AbsoluteSpan, Offset, Span, Spanned},
};

/// Macro to add `impl From<X> for Y` to enums.
/// Couldn't find a derive for this (!).
macro_rules! add_from_impls {
    ($(#[$attr:meta])* $v:vis enum $name:ident<$lt:lifetime> { $(
        $(#[$variant_meta:meta])*
        $variant:ident($variant_ty:ty),)*
    }) => {
        $(#[$attr])*
        $v enum $name<$lt> {
            $(
                $(#[$variant_meta])*
                $variant($variant_ty),
            )*
        }

        $(
            impl<$lt> From<$variant_ty> for $name<$lt> {
                fn from(v: $variant_ty) -> Self {
                    $name::$variant(v)
                }
            }
        )*
    };
}

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

#[salsa::tracked]
pub struct Module<'db> {
    #[return_ref]
    pub items: AstVec<'db, Item<'db>>,
}

impl<'db> Spanned<'db> for Module<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.items(db).span
    }
}

add_from_impls! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update)]
    pub enum Item<'db> {
        SourceFile(SourceFile),
        Use(UseItem<'db>),
        Class(ClassItem<'db>),
        Function(Function<'db>),
    }
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
            Item::Function(data) => data.span(db),
        }
    }

    pub fn absolute_span(&self, db: &'db dyn crate::Db) -> AbsoluteSpan {
        match self {
            Item::SourceFile(source_file) => source_file.absolute_span(db),
            Item::Use(data) => data.span(db).absolute_span(db),
            Item::Class(data) => data.span(db).absolute_span(db),
            Item::Function(data) => data.span(db).absolute_span(db),
        }
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

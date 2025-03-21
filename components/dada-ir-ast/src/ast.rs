#![allow(clippy::unused_unit)] // FIXME: derive(Update) triggers these

use dada_util::FromImpls;
use dada_util::SalsaSerialize;
use salsa::Update;
use serde::Serialize;

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

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub struct Identifier<'db> {
    #[return_ref]
    pub text: String,
}

impl<'db> Identifier<'db> {
    pub fn prelude(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "prelude")
    }

    pub fn dada(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "dada")
    }

    pub fn main(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "main")
    }

    pub fn new_ident(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "new")
    }

    /// Create interned "self" identifier
    pub fn self_ident(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "self")
    }

    /// Create interned "Self" identifier
    pub fn self_ty_ident(db: &'db dyn crate::Db) -> Identifier<'db> {
        Identifier::new(db, "Self")
    }
}

impl std::fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| write!(f, "{}", self.text(db)))
            .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct AstModule<'db> {
    pub name: Identifier<'db>,

    #[return_ref]
    pub items: SpanVec<'db, AstItem<'db>>,
}

impl<'db> Spanned<'db> for AstModule<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.items(db).span
    }
}

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update, FromImpls, Serialize,
)]
pub enum AstItem<'db> {
    SourceFile(SourceFile),
    Use(AstUse<'db>),
    Aggregate(AstAggregate<'db>),
    Function(AstFunction<'db>),
}

/// A "path" identifies an item and a partial set of substitutions.
#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct AstPath<'db> {
    #[return_ref]
    pub kind: AstPathKind<'db>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update, Serialize)]
pub enum AstPathKind<'db> {
    /// `$id` that starts a path
    Identifier(SpannedIdentifier<'db>),

    /// `$path[...]`
    GenericArgs {
        path: AstPath<'db>,
        args: SpanVec<'db, AstGenericTerm<'db>>,
    },

    /// `$path.$id`
    Member {
        path: AstPath<'db>,
        id: SpannedIdentifier<'db>,
    },
}

impl<'db> AstPath<'db> {
    pub fn len(self, db: &'db dyn crate::Db) -> usize {
        match self.kind(db) {
            AstPathKind::Identifier(_) => 1,
            AstPathKind::GenericArgs { path, .. } => path.len(db) + 1,
            AstPathKind::Member { path, .. } => path.len(db) + 1,
        }
    }

    pub fn first_id(self, db: &'db dyn crate::Db) -> SpannedIdentifier<'db> {
        match *self.kind(db) {
            AstPathKind::Identifier(id) => id,
            AstPathKind::GenericArgs { path, .. } => path.first_id(db),
            AstPathKind::Member { path, .. } => path.first_id(db),
        }
    }

    pub fn last_id(self, db: &'db dyn crate::Db) -> SpannedIdentifier<'db> {
        match *self.kind(db) {
            AstPathKind::Identifier(id) => id,
            AstPathKind::GenericArgs { path, .. } => path.first_id(db),
            AstPathKind::Member { id: ident, .. } => ident,
        }
    }
}

impl<'db> Spanned<'db> for AstPath<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self.kind(db) {
            AstPathKind::Identifier(id) => id.span,
            AstPathKind::GenericArgs { path, args } => path.first_id(db).span.to(db, args.span),
            AstPathKind::Member { path, id: ident } => path.first_id(db).span.to(db, ident.span),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub struct SpannedIdentifier<'db> {
    pub span: Span<'db>,
    pub id: Identifier<'db>,
}

impl<'db> Spanned<'db> for SpannedIdentifier<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

/// For functions, classes, and other items we often defer parsing their contents.
/// This struct captures the contents and the span at which they appeared.
/// It can then be used to parse the contents later.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub struct DeferredParse<'db> {
    pub span: Span<'db>,
    pub contents: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub struct AstVisibility<'db> {
    pub span: Span<'db>,
    pub kind: VisibilityKind,
}

impl<'db> Spanned<'db> for AstVisibility<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum VisibilityKind {
    Export,
    Pub,
}

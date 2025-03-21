use dada_util::SalsaSerialize;
use serde::Serialize;

use crate::{
    ast::{AstFieldDecl, AstVisibility, DeferredParse},
    span::{Span, Spanned},
};

use super::{AstGenericDecl, Identifier, SpanVec};

/// Some kind of aggregate, like a class, struct, etc.
///
/// `class $name[$generics] { ... }` or `class $name[$generics](...) { ... }`
#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct AstAggregate<'db> {
    pub span: Span<'db>,

    /// Visibility of the class
    pub visibility: Option<AstVisibility<'db>>,

    pub kind: AstAggregateKind,

    pub name: Identifier<'db>,

    pub name_span: Span<'db>,

    #[return_ref]
    pub generics: Option<SpanVec<'db, AstGenericDecl<'db>>>,

    /// If a `()` section is present...
    #[return_ref]
    pub inputs: Option<SpanVec<'db, AstFieldDecl<'db>>>,

    /// The unparsed contents of the class.
    /// This can be parsed via the `members`
    /// method defined in `dada_parser::prelude`.
    #[return_ref]
    pub contents: Option<DeferredParse<'db>>,
}

impl<'db> Spanned<'db> for AstAggregate<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstAggregate::span(*self, db)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub enum AstAggregateKind {
    Class,
    Struct,
}

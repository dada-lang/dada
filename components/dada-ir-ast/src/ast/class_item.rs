use crate::{
    ast::{AstFieldDecl, AstVisibility, DeferredParse},
    span::{Span, Spanned},
};

use super::{AstGenericDecl, Identifier, SpanVec};

/// `class $name[$generics] { ... }` or `class $name[$generics](...) { ... }`
#[salsa::tracked]
pub struct AstClassItem<'db> {
    pub span: Span<'db>,

    /// Visibility of the class
    pub visibility: Option<AstVisibility<'db>>,

    #[id]
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

impl<'db> AstClassItem<'db> {}

impl<'db> Spanned<'db> for AstClassItem<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstClassItem::span(*self, db)
    }
}

use dada_util::FromImpls;
use salsa::Update;

use crate::span::{Span, Spanned};

use super::{AstPath, SpanVec, SpannedIdentifier};

#[salsa::tracked]
pub struct AstTy<'db> {
    pub span: Span<'db>,
    pub kind: AstTyKind<'db>,
}

impl<'db> Spanned<'db> for AstTy<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstTy::span(*self, db)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstTyKind<'db> {
    /// `$Perm $Ty`, e.g., `shared String`
    Perm(AstPerm<'db>, AstTy<'db>),

    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    Named(AstPath<'db>, Option<SpanVec<'db, AstGenericArg<'db>>>),

    /// `type T`
    GenericDecl(AstGenericDecl<'db>),

    /// `?`
    Unknown,
}

#[salsa::tracked]
pub struct AstPerm<'db> {
    pub span: Span<'db>,

    #[return_ref]
    pub kind: AstPermKind<'db>,
}

impl<'db> Spanned<'db> for AstPerm<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstPerm::span(*self, db)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstPermKind<'db> {
    Shared(Option<SpanVec<'db, AstPath<'db>>>),
    Leased(Option<SpanVec<'db, AstPath<'db>>>),
    Given(Option<SpanVec<'db, AstPath<'db>>>),
    My,
    Our,
    Variable(SpannedIdentifier<'db>),

    /// `perm P`
    GenericDecl(AstGenericDecl<'db>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum AstGenericArg<'db> {
    /// Something clearly a type
    Ty(AstTy<'db>),

    /// Something clearly a permission
    Perm(AstPerm<'db>),

    /// A single identifier is ambiguous and must be disambiguated by the type checker
    Id(SpannedIdentifier<'db>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstGenericKind<'db> {
    Type(Span<'db>),
    Perm(Span<'db>),
}

impl<'db> Spanned<'db> for AstGenericKind<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstGenericKind::Type(span) => *span,
            AstGenericKind::Perm(span) => *span,
        }
    }
}

/// `type T? (: bounds)?`
/// `perm T? (: bounds)?`
#[salsa::tracked]
pub struct AstGenericDecl<'db> {
    pub kind: AstGenericKind<'db>,
    pub name: Option<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for AstGenericDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        if let Some(name) = self.name(db) {
            self.kind(db).span(db).to(name.span(db))
        } else {
            self.kind(db).span(db)
        }
    }
}

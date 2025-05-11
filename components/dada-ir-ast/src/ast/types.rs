use dada_util::{FromImpls, SalsaSerialize};
use salsa::Update;
use serde::Serialize;

use crate::span::{Span, Spanned};

use super::{AstPath, SpanVec, SpannedIdentifier};

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct AstTy<'db> {
    pub span: Span<'db>,
    pub kind: AstTyKind<'db>,
}

impl<'db> Spanned<'db> for AstTy<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstTy::span(*self, db)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum AstTyKind<'db> {
    /// `$Perm $Ty`, e.g., `shared String`
    Perm(AstPerm<'db>, AstTy<'db>),

    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    Named(AstPath<'db>, Option<SpanVec<'db, AstGenericTerm<'db>>>),

    /// `type T`
    GenericDecl(AstGenericDecl<'db>),
}

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum AstPermKind<'db> {
    /// User wrote `ref` or `ref[place1, place2]`
    Referenced(Option<SpanVec<'db, AstPath<'db>>>),

    /// User wrote `leased` or `leased[place1, place2]`
    Leased(Option<SpanVec<'db, AstPath<'db>>>),

    /// User wrote `given` or `given[place1, place2]`
    Given(Option<SpanVec<'db, AstPath<'db>>>),

    /// User wrote `my`
    My,

    /// User wrote `our`
    Our,

    /// User wrote `P`
    Variable(SpannedIdentifier<'db>),

    /// User wrote `perm P`
    GenericDecl(AstGenericDecl<'db>),
}

#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls, Serialize,
)]
pub enum AstGenericTerm<'db> {
    /// Something clearly a type
    Ty(AstTy<'db>),

    /// Something clearly a permission
    Perm(AstPerm<'db>),

    /// A single identifier is ambiguous and must be disambiguated by the type checker
    Id(SpannedIdentifier<'db>),
}

impl<'db> Spanned<'db> for AstGenericTerm<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstGenericTerm::Ty(ty) => ty.span(db),
            AstGenericTerm::Perm(perm) => perm.span(db),
            AstGenericTerm::Id(id) => id.span(db),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
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
#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct AstGenericDecl<'db> {
    pub kind: AstGenericKind<'db>,
    pub name: Option<SpannedIdentifier<'db>>,
}

impl<'db> Spanned<'db> for AstGenericDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        if let Some(name) = self.name(db) {
            self.kind(db).span(db).to(db, name.span(db))
        } else {
            self.kind(db).span(db)
        }
    }
}

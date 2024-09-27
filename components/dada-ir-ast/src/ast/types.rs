use salsa::Update;

use crate::span::{Span, Spanned};

use super::{AstPath, Identifier, SpanVec, SpannedIdentifier};

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
    GenericDecl {
        keyword_span: Span<'db>,
        decl: KindedGenericDecl<'db>,
    },

    /// `?`
    Unknown,
}

#[salsa::interned] // (*)
pub struct AstPerm<'db> {
    pub span: Span<'db>,
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
    Variable(Identifier<'db>),

    /// `perm P`
    GenericDecl {
        keyword_span: Span<'db>,
        decl: KindedGenericDecl<'db>,
    },
}

add_from_impls! {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
    pub enum AstGenericArg<'db> {
        /// Something clearly a type
        Ty(AstTy<'db>),

        /// Something clearly a permission
        Perm(AstPerm<'db>),

        /// A single identifier is ambiguous and must be disambiguated by the type checker
        Id(SpannedIdentifier<'db>),
    }
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

#[salsa::tracked]
pub struct AstGenericDecl<'db> {
    pub kind: AstGenericKind<'db>,
    pub decl: KindedGenericDecl<'db>,
}

impl<'db> Spanned<'db> for AstGenericDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.kind(db).span(db).to(self.decl(db).span(db))
    }
}

/// `[type T]` or `[perm P]`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct KindedGenericDecl<'db> {
    pub name: SpannedIdentifier<'db>,
}

impl<'db> Spanned<'db> for KindedGenericDecl<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.name.span
    }
}

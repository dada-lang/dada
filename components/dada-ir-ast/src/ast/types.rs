use salsa::{DebugWithDb, Update};

use crate::span::{Span, Spanned};

use super::{AstVec, Identifier, Path, SpannedIdentifier};

// (*) Interned isn't really what we want here. We really want something like `#[salsa::boxed]`
// that will cheaply allocate the thing. But I'm trying to push on our existing salsa types
// to see how it works.

#[salsa::interned] // (*)
pub struct AstTy<'db> {
    pub span: Span<'db>,
    pub kind: AstTyKind<'db>,
}

impl<'db> Spanned<'db> for AstTy<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstTy::span(*self, db)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstTyKind<'db> {
    /// `$Perm $Ty`, e.g., `shared String`
    Perm(AstPerm<'db>, AstTy<'db>),

    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    Named(Path<'db>, Option<AstVec<'db, AstGenericArg<'db>>>),

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstPermKind<'db> {
    Shared(Option<AstVec<'db, Path<'db>>>),
    Leased(Option<AstVec<'db, Path<'db>>>),
    Given(Option<AstVec<'db, Path<'db>>>),
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
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
    pub enum AstGenericArg<'db> {
        /// Something clearly a type
        Ty(AstTy<'db>),

        /// Something clearly a permission
        Perm(AstPerm<'db>),

        /// A single identifier is ambiguous and must be disambiguated by the type checker
        Id(SpannedIdentifier<'db>),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct GenericDecl<'db> {
    pub kind: AstGenericKind<'db>,
    pub decl: KindedGenericDecl<'db>,
}

impl<'db> Spanned<'db> for GenericDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.kind.span(db).to(self.decl.span(db))
    }
}

/// `[type T]` or `[perm P]`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct KindedGenericDecl<'db> {
    pub name: SpannedIdentifier<'db>,
}

impl<'db> Spanned<'db> for KindedGenericDecl<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.name.span
    }
}

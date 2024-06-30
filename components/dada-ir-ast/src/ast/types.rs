use salsa::{DebugWithDb, Update};

use crate::span::Span;

use super::{AstVec, Path, SpannedIdentifier};

// (*) Interned isn't really what we want here. We really want something like `#[salsa::boxed]`
// that will cheaply allocate the thing. But I'm trying to push on our existing salsa types
// to see how it works.

#[salsa::interned] // (*)
pub struct AstTy<'db> {
    pub span: Span<'db>,
    pub kind: AstTyKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstTyKind<'db> {
    /// `perm ty`, e.g., `shared String`
    Perm(AstPerm<'db>, AstTy<'db>),

    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    Named(Path<'db>, AstVec<'db, AstGenericArg<'db>>),

    /// `type T`
    GenericDecl(KindedGenericDecl<'db>),

    /// `?`
    Unknown,
}

#[salsa::interned] // (*)
pub struct AstPerm<'db> {
    pub span: Span<'db>,
    pub kind: AstPermKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstPermKind<'db> {
    Shared(AstVec<'db, Path<'db>>),
    Leased(AstVec<'db, Path<'db>>),
    Given(AstVec<'db, Path<'db>>),
    My,
    Our,
    Variable(SpannedIdentifier<'db>),

    /// `perm P`
    GenericDecl(KindedGenericDecl<'db>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstGenericArg<'db> {
    Ty(AstTy<'db>),
    Perm(AstPerm<'db>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum GenericKind {
    Type,
    Perm,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum GenericDecl<'db> {
    /// User just wrote `T`
    Default(SpannedIdentifier<'db>),

    /// User wrote `type T` or `perm T`, etc
    Kinded(GenericKind, KindedGenericDecl<'db>),
}

/// `[type T]` or `[perm P]`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct KindedGenericDecl<'db> {
    pub keyword_span: Span<'db>,
    pub name: SpannedIdentifier<'db>,
}

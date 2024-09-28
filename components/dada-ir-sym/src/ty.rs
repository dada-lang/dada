use dada_ir_ast::{ast::Identifier, span::Span};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymClass,
    indices::{SymBinderIndex, SymBoundVarIndex, SymExistentialVarIndex, SymUniversalVarIndex},
    symbol::{SymField, SymGeneric, SymLocalVariable},
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Update, Debug)]
pub enum SymGenericKind {
    Type,
    Perm,
}

/// Value of a generic parameter
#[salsa::tracked]
pub struct SymGenericDecl<'db> {
    pub kind: SymGenericKind,
    pub name: Option<Identifier<'db>>,
    pub span: Span<'db>,
}

/// Value of a generic parameter
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymGenericArg<'db> {
    Type(SymTy<'db>),
    Perm(SymPerm<'db>),
}

#[salsa::interned]
pub struct SymTy<'db> {
    pub kind: SymTyKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymTyKind<'db> {
    Perm(SymPerm<'db>, SymTy<'db>),

    Named(SymTyName<'db>, Vec<SymGenericArg<'db>>),

    FreeUniversal(SymUniversalVarIndex),

    FreeExistential(SymExistentialVarIndex),

    BoundVar(SymBinderIndex, SymBoundVarIndex),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymTyName<'db> {
    Class(SymClass<'db>),
}

#[salsa::interned]
pub struct SymPerm<'db> {
    pub kind: PermKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum PermKind<'db> {
    My,
    Our,
    Generic(SymGeneric<'db>),
    Shared(Vec<SymPlace<'db>>),
    Leased(Vec<SymPlace<'db>>),
    Given(Vec<SymPlace<'db>>),
}

#[salsa::tracked]
pub struct SymPlace<'db> {
    pub kind: SymPlaceKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPlaceKind<'db> {
    LocalVariable(SymLocalVariable<'db>),
    Field(SymPlace<'db>, SymField<'db>),
    Index,
}

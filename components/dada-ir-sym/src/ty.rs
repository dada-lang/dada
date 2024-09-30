use dada_ir_ast::{ast::Identifier, diagnostic::Reported, span::Span};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymClass,
    indices::{SymBinderIndex, SymBoundVarIndex, SymExistentialVarIndex, SymUniversalVarIndex},
    symbol::{SymField, SymGeneric, SymLocalVariable},
};

/// Value of a generic parameter
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymGenericArg<'db> {
    Type(SymTy<'db>),
    Perm(SymPerm<'db>),
    Error(Reported),
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

    /// Indicates the user wrote `?` and we should use gradual typing.
    Unknown,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymTyName<'db> {
    Class(SymClass<'db>),

    #[no_from_impl]
    Tuple {
        arity: usize,
    },
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

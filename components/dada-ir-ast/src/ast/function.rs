use salsa::{DebugWithDb, Update};

use super::{AstBlock, AstPerm, AstTy, AstVec, GenericDecl, SpannedIdentifier};
use crate::span::Span;

#[salsa::tracked]
pub struct Function<'db> {
    /// Span of the `fn` keyword
    pub fn_span: Span<'db>,

    /// Name of the function
    pub name: SpannedIdentifier<'db>,

    /// Any explicit generics e.g., `[type T]`
    pub generics: AstVec<'db, GenericDecl<'db>>,

    /// Arguments to the function
    pub arguments: AstVec<'db, AstFunctionArg<'db>>,

    /// Return type of the function (if provided)
    pub return_ty: Option<AstTy<'db>>,

    /// Body (if provided)
    pub body: Option<AstBlock<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstFunctionArg<'db> {
    SelfArg(AstSelfArg<'db>),
    Variable(VariableDecl<'db>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstSelfArg<'db> {
    perm: Option<AstPerm<'db>>,
    self_span: Span<'db>,
}

/// `x: T`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct VariableDecl<'db> {
    pub name: SpannedIdentifier<'db>,
    pub ty: AstTy<'db>,
}

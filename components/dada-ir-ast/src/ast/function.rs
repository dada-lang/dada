use dada_util::FromImpls;
use salsa::Update;

use super::{AstGenericDecl, AstPerm, AstTy, SpanVec, SpannedIdentifier};
use crate::{
    ast::{AstVisibility, DeferredParse},
    span::{Span, Spanned},
};

/// `fn foo() { }`
#[salsa::tracked]
pub struct AstFunction<'db> {
    /// Overall span of the function declaration
    pub span: Span<'db>,

    /// Declared effects (e.g., `async`)
    pub effects: AstFunctionEffects<'db>,

    /// Span of the `fn` keyword
    pub fn_span: Span<'db>,

    /// Visibility of the function
    pub visibility: Option<AstVisibility<'db>>,

    /// Name of the function
    pub name: SpannedIdentifier<'db>,

    /// Any explicit generics e.g., `[type T]`
    #[return_ref]
    pub generics: Option<SpanVec<'db, AstGenericDecl<'db>>>,

    /// Arguments to the function
    #[return_ref]
    pub inputs: SpanVec<'db, AstFunctionInput<'db>>,

    /// Return type of the function (if provided)
    pub output_ty: Option<AstTy<'db>>,

    /// Body (if provided)
    #[return_ref]
    pub body: Option<DeferredParse<'db>>,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstFunctionEffects<'db> {
    pub async_effect: Option<Span<'db>>,
    pub unsafe_effect: Option<Span<'db>>,
}

impl<'db> Spanned<'db> for AstFunction<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstFunction::span(*self, db)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum AstFunctionInput<'db> {
    SelfArg(AstSelfArg<'db>),
    Variable(VariableDecl<'db>),
}

impl<'db> Spanned<'db> for AstFunctionInput<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstFunctionInput::SelfArg(arg) => arg.span(db),
            AstFunctionInput::Variable(var) => var.span(db),
        }
    }
}

#[salsa::tracked]
pub struct AstSelfArg<'db> {
    pub perm: AstPerm<'db>,
    pub self_span: Span<'db>,
}

impl<'db> Spanned<'db> for AstSelfArg<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.self_span(db).start_from(self.perm(db).span(db))
    }
}

/// `[mut] x: T`
#[salsa::tracked]
pub struct VariableDecl<'db> {
    /// Span of the `mut` keyword, if present.
    pub mutable: Option<Span<'db>>,

    /// Variable name.
    pub name: SpannedIdentifier<'db>,

    /// Variable type.
    pub ty: AstTy<'db>,
}

impl<'db> Spanned<'db> for VariableDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.name(db).span.to(db, self.ty(db).span(db))
    }
}

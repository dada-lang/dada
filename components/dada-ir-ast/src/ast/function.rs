use dada_util::FromImpls;
use salsa::Update;

use super::{AstGenericDecl, AstPerm, AstTy, SpanVec, SpannedIdentifier};
use crate::span::{Span, Spanned};

/// `fn foo() { }`
#[salsa::tracked]
pub struct AstFunction<'db> {
    /// Overall span of the function declaration
    pub span: Span<'db>,

    /// Span of the `fn` keyword
    pub fn_span: Span<'db>,

    /// Name of the function
    pub name: SpannedIdentifier<'db>,

    /// Any explicit generics e.g., `[type T]`
    #[return_ref]
    pub generics: Option<SpanVec<'db, AstGenericDecl<'db>>>,

    /// Arguments to the function
    #[return_ref]
    pub arguments: SpanVec<'db, AstFunctionArg<'db>>,

    /// Return type of the function (if provided)
    pub return_ty: Option<AstTy<'db>>,

    /// Body (if provided)
    pub body: Option<AstFunctionBody<'db>>,
}

impl<'db> Spanned<'db> for AstFunction<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstFunction::span(*self, db)
    }
}

#[salsa::tracked]
pub struct AstFunctionBody<'db> {
    pub span: Span<'db>,

    #[return_ref]
    pub contents: String,
}

impl<'db> Spanned<'db> for AstFunctionBody<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstFunctionBody::span(*self, db)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum AstFunctionArg<'db> {
    SelfArg(AstSelfArg<'db>),
    Variable(VariableDecl<'db>),
}

impl<'db> Spanned<'db> for AstFunctionArg<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstFunctionArg::SelfArg(arg) => arg.span(db),
            AstFunctionArg::Variable(var) => var.span(db),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstSelfArg<'db> {
    pub perm: Option<AstPerm<'db>>,
    pub self_span: Span<'db>,
}

impl<'db> Spanned<'db> for AstSelfArg<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.self_span.start_from(self.perm.map(|p| p.span(db)))
    }
}

/// `x: T`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct VariableDecl<'db> {
    pub name: SpannedIdentifier<'db>,
    pub ty: AstTy<'db>,
}

impl<'db> Spanned<'db> for VariableDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.name.span.to(self.ty.span(db))
    }
}

use salsa::Update;

use super::{AstPerm, AstTy, AstVec, GenericDecl, SpannedIdentifier};
use crate::span::{Span, Spanned};

#[salsa::tracked]
pub struct Function<'db> {
    /// Overall span of the function declaration
    pub span: Span<'db>,

    /// Span of the `fn` keyword
    pub fn_span: Span<'db>,

    /// Name of the function
    pub name: SpannedIdentifier<'db>,

    /// Any explicit generics e.g., `[type T]`
    pub generics: Option<AstVec<'db, GenericDecl<'db>>>,

    /// Arguments to the function
    pub arguments: AstVec<'db, AstFunctionArg<'db>>,

    /// Return type of the function (if provided)
    pub return_ty: Option<AstTy<'db>>,

    /// Body (if provided)
    pub body: Option<FunctionBody<'db>>,
}

impl<'db> Spanned<'db> for Function<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        Function::span(*self, db)
    }
}

#[salsa::tracked]
pub struct FunctionBody<'db> {
    pub span: Span<'db>,

    #[return_ref]
    pub contents: String,
}

impl<'db> Spanned<'db> for FunctionBody<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        FunctionBody::span(*self, db)
    }
}

add_from_impls! {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
    pub enum AstFunctionArg<'db> {
        SelfArg(AstSelfArg<'db>),
        Variable(VariableDecl<'db>),
    }
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

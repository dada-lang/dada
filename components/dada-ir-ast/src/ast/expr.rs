use salsa::Update;

use crate::span::Span;

use super::{AstGenericArg, AstTy, AstVec, Path, SpannedIdentifier};

#[salsa::tracked]
pub struct AstBlock<'db> {
    statements: AstVec<'db, AstStatement<'db>>,
}

add_from_impls! {
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstStatement<'db> {
    Let(AstLetStatement<'db>),
    Expr(AstExpr<'db>),
}
}

/// `let x = v`, `let x: t = v`, etc
#[salsa::tracked]
pub struct AstLetStatement<'db> {
    pub name: SpannedIdentifier<'db>,
    pub ty: Option<AstTy<'db>>,
    pub initializer: Option<AstExpr<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstExpr<'db> {
    pub span: Span<'db>,
    pub kind: Box<AstExprKind<'db>>,
}

impl<'db> AstExpr<'db> {
    pub fn new(span: Span<'db>, kind: AstExprKind<'db>) -> Self {
        Self {
            span,
            kind: Box::new(kind),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstExprKind<'db> {
    /// `22`
    Literal(Literal<'db>),

    /// `a`, `a.b`, etc
    Path(Path<'db>),

    /// `f(a, b, c)`
    Call(AstCallExpr<'db>),

    /// `(a, b, c)`
    Tuple(AstVec<'db, AstExpr<'db>>),

    /// `a { field: value }`
    Constructor(Path<'db>, AstVec<'db, AstConstructorField<'db>>),

    /// `return x`
    Return(Option<AstExpr<'db>>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstConstructorField<'db> {
    pub name: SpannedIdentifier<'db>,
    pub value: AstExpr<'db>,
}

#[salsa::interned]
pub struct Literal<'db> {
    kind: LiteralKind,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum LiteralKind {
    Integer,
    String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstCallExpr<'db> {
    pub callee: AstExpr<'db>,
    pub generic_args: Option<AstVec<'db, AstGenericArg<'db>>>,
    pub args: AstVec<'db, AstExpr<'db>>,
}

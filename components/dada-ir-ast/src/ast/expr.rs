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

    /// `x`
    Id(SpannedIdentifier<'db>),

    /// `E.f`
    ///
    /// Note that this is not necessarily a field.
    /// Interpretation is needed.
    DotId(AstExpr<'db>, SpannedIdentifier<'db>),

    /// `E[..]`
    ///
    /// Note that we cannot parse the contents of the `[..]`
    /// until we have resolved the expression `E`.
    SquareBracketOp(AstExpr<'db>, SquareBracketArgs<'db>),

    /// `E(expr0, expr1, ..., exprN)`
    ///
    /// Note that the callee expression could also be
    /// a `DotId` in which case this is a method call
    /// as well as a `SquareBracketsOp`.
    ParenthesisOp(AstExpr<'db>, AstVec<'db, AstExpr<'db>>),

    /// `(a, b, c)`
    ///
    /// Could also be `(a)`.
    Tuple(AstVec<'db, AstExpr<'db>>),

    /// `a { field: value }`
    Constructor(Path<'db>, AstVec<'db, AstConstructorField<'db>>),

    /// `return x`
    Return(Option<AstExpr<'db>>),

    BinaryOp(BinaryOp, AstExpr<'db>, AstExpr<'db>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Created when we parse a `x[..]` expression to store the `..` contents.
/// We can't eagerly parse it because we don't yet know whether to parse it
/// as types or expressions.
#[salsa::tracked]
pub struct SquareBracketArgs<'db> {
    span: Span<'db>,

    #[return_ref]
    text: String,
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
pub struct AstParenExpr<'db> {
    pub callee: AstExpr<'db>,
    pub generic_args: Option<AstVec<'db, AstGenericArg<'db>>>,
    pub args: AstVec<'db, AstExpr<'db>>,
}

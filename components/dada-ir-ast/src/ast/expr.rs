use dada_util::FromImpls;
use salsa::Update;

use crate::span::Span;

use super::{AstGenericTerm, AstPath, AstTy, DeferredParse, SpanVec, SpannedIdentifier};

#[salsa::tracked]
pub struct AstBlock<'db> {
    #[return_ref]
    pub statements: SpanVec<'db, AstStatement<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum AstStatement<'db> {
    Let(AstLetStatement<'db>),
    Expr(AstExpr<'db>),
}

/// `let x = v`, `let x: t = v`, etc
#[salsa::tracked]
pub struct AstLetStatement<'db> {
    pub mutable: Option<Span<'db>>,
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
    /// `{ ... }`
    Block(AstBlock<'db>),

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
    ParenthesisOp(AstExpr<'db>, SpanVec<'db, AstExpr<'db>>),

    /// `(a, b, c)`
    ///
    /// Could also be `(a)`.
    Tuple(SpanVec<'db, AstExpr<'db>>),

    /// `a { field: value }`
    Constructor(AstPath<'db>, SpanVec<'db, AstConstructorField<'db>>),

    /// `return x`
    Return(Option<AstExpr<'db>>),

    /// `x.await`
    Await {
        future: AstExpr<'db>,
        await_keyword: Span<'db>,
    },

    /// `x.lease`, `x.share`, or `x.give`
    PermissionOp {
        value: AstExpr<'db>,
        op: PermissionOp,
    },

    /// `a + b` etc
    BinaryOp(SpannedBinaryOp<'db>, AstExpr<'db>, AstExpr<'db>),

    /// `!foo` etc
    UnaryOp(SpannedUnaryOp<'db>, AstExpr<'db>),

    /// If/else-if chain
    If(Vec<IfArm<'db>>),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum PermissionOp {
    Lease,
    Share,
    Give,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct IfArm<'db> {
    /// if None, this is an `else` (and should come last)
    pub condition: Option<AstExpr<'db>>,

    /// the value
    pub result: AstBlock<'db>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SpannedBinaryOp<'db> {
    pub span: Span<'db>,
    pub op: AstBinaryOp,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AstBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    AndAnd,
    OrOr,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    Assign,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SpannedUnaryOp<'db> {
    pub span: Span<'db>,
    pub op: UnaryOp,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum UnaryOp {
    Not,
    Negate,
}

/// Created when we parse `x[..]` expressions or paths to store the `..` contents.
/// We can't eagerly parse it because we don't yet know whether to parse it
/// as types or expressions.
#[salsa::tracked]
pub struct SquareBracketArgs<'db> {
    #[return_ref]
    pub deferred: DeferredParse<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstConstructorField<'db> {
    pub name: SpannedIdentifier<'db>,
    pub value: AstExpr<'db>,
}

#[salsa::interned]
pub struct Literal<'db> {
    pub kind: LiteralKind,
    #[return_ref]
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum LiteralKind {
    Boolean,
    Integer,
    String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstParenExpr<'db> {
    pub callee: AstExpr<'db>,
    pub generic_args: Option<SpanVec<'db, AstGenericTerm<'db>>>,
    pub args: SpanVec<'db, AstExpr<'db>>,
}

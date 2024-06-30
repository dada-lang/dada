use salsa::{DebugWithDb, Update};

use crate::span::Span;

use super::{AstGenericArg, AstTy, AstVec, Path, SpannedIdentifier};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstBlock<'db> {
    statements: AstVec<'db, AstStatement<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum AstStatement<'db> {
    Let(AstLetStatement<'db>),
    Expr(AstExpr<'db>),
}

/// `let x = v`, `let x: t = v`, etc
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstLetStatement<'db> {
    pub name: SpannedIdentifier<'db>,
    pub ty: Option<AstTy<'db>>,
    pub initializer: Option<AstExpr<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstExpr<'db> {
    pub span: Span<'db>,
    pub kind: Box<AstExprKind<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstConstructorField<'db> {
    name: SpannedIdentifier<'db>,
    value: AstExpr<'db>,
}

#[salsa::interned]
pub struct Literal<'db> {
    kind: LiteralKind,
    text: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum LiteralKind {
    Integer,
    String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct AstCallExpr<'db> {
    pub callee: AstExpr<'db>,
    pub generic_args: AstVec<'db, AstGenericArg<'db>>,
    pub args: AstVec<'db, AstExpr<'db>>,
}

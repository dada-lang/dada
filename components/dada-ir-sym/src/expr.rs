use dada_ir_ast::{
    ast::{BinaryOp, Literal, SpanVec},
    span::Span,
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    symbol::{SymField, SymLocalVariable},
    ty::{SymGenericArg, SymTy},
};

#[salsa::tracked]
struct SymBlock<'db> {
    statements: SpanVec<'db, SymStatement<'db>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymStatement<'db> {
    Let(SymLetStatement<'db>),
    Expr(SymExpr<'db>),
}

/// `let x = v`, `let x: t = v`, etc
#[salsa::tracked]
pub struct SymLetStatement<'db> {
    pub variable: SymLocalVariable<'db>,
    pub initializer: Option<SymExpr<'db>>,
}

#[salsa::tracked]
pub struct SymExpr<'db> {
    pub span: Span<'db>,
    pub kind: SymExprKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymExprKind<'db> {
    /// `22`
    Literal(Literal<'db>),

    /// `$place.share`
    Share(SymPlaceExpr<'db>),

    /// `$place.lease`
    Lease(SymPlaceExpr<'db>),

    /// `$place.give`
    Give(SymPlaceExpr<'db>),

    /// `$expr.method[g1, g2](a1, a2)`
    MethodCall(SymMethodCall<'db>),

    /// `()`
    Unit,

    /// `(a, b, c)`
    ///
    /// Length of vector must be at least 2.
    Tuple(Vec<SymExpr<'db>>),

    /// `Foo { field: value }`
    Constructor(SymConstructor<'db>),

    /// `return x`
    Return(Option<SymExpr<'db>>),

    /// `a + b`
    BinaryOp(BinaryOp, SymExpr<'db>, SymExpr<'db>),
}

/// `$expr.method[g1, g2](a1, a2)`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymMethodCall<'db> {
    pub owner: SymExpr<'db>,
    pub generic_args: Vec<SymGenericArg<'db>>,
    pub args: Vec<SymExpr<'db>>,
}

/// `a[g1, g2] { field: value }`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymConstructor<'db> {
    pub ty: SymTy<'db>,
    pub fields: Vec<SymConstructorField<'db>>,
}

/// `field: value`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymConstructorField<'db> {
    pub field: SymField<'db>,
    pub initializer: SymExpr<'db>,
}

#[salsa::tracked]
pub struct SymPlaceExpr<'db> {
    pub span: Span<'db>,
    pub kind: SymPlaceExprKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPlaceExprKind<'db> {
    LocalVariable(SymLocalVariable<'db>),
    Field(SymPlaceExpr<'db>, SymField<'db>),
    Index(SymExpr<'db>),
}

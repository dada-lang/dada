use dada_ir_ast::{
    ast::{BinaryOp, Literal},
    diagnostic::Reported,
    span::Span,
};
use dada_ir_sym::{
    class::SymField,
    symbol::SymLocalVariable,
    ty::{SymGenericTerm, SymTy},
};
use salsa::Update;

#[salsa::tracked]
pub struct CheckedExpr<'db> {
    pub span: Span<'db>,
    pub kind: CheckedExprKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum CheckedExprKind<'db> {
    /// `$expr1; $expr2`
    Semi(CheckedExpr<'db>, CheckedExpr<'db>),

    /// `let $lv: $ty = $expr in $expr`
    Let(CheckedLetExpr<'db>),

    /// `22`
    Literal(Literal<'db>),

    /// `$place.share`
    Share(CheckedPlaceExpr<'db>),

    /// `$place.lease`
    Lease(CheckedPlaceExpr<'db>),

    /// `$place.give`
    Give(CheckedPlaceExpr<'db>),

    /// `$expr.method[g1, g2](a1, a2)`
    MethodCall(CheckedMethodCall<'db>),

    /// `()`
    Unit,

    /// `(a, b, c)`
    ///
    /// Length of vector must be at least 2.
    Tuple(Vec<CheckedExpr<'db>>),

    /// `Foo { field: value }`
    Constructor(CheckedConstructor<'db>),

    /// `return x`
    Return(Option<CheckedExpr<'db>>),

    /// `a + b`
    BinaryOp(BinaryOp, CheckedExpr<'db>, CheckedExpr<'db>),
}

/// `let $lv: $ty = $initializer in $body`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct CheckedLetExpr<'db> {
    pub lv: SymLocalVariable<'db>,
    pub ty: SymTy<'db>,
    pub initializer: Option<CheckedExpr<'db>>,
    pub body: CheckedExpr<'db>,
}

/// `$expr.method[g1, g2](a1, a2)`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct CheckedMethodCall<'db> {
    pub owner: CheckedExpr<'db>,
    pub generic_args: Vec<SymGenericTerm<'db>>,
    pub args: Vec<CheckedExpr<'db>>,
}

/// `a[g1, g2] { field: value }`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct CheckedConstructor<'db> {
    pub ty: SymTy<'db>,
    pub fields: Vec<CheckedConstructorField<'db>>,
}

/// `field: value`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct CheckedConstructorField<'db> {
    pub field: SymField<'db>,
    pub initializer: CheckedExpr<'db>,
}

#[salsa::tracked]
pub struct CheckedPlaceExpr<'db> {
    pub span: Span<'db>,
    pub kind: CheckedPlaceExprKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum CheckedPlaceExprKind<'db> {
    /// `x`
    LocalVariable(SymLocalVariable<'db>),

    /// `x.f`
    Field(CheckedPlaceExpr<'db>, SymField<'db>),

    /// `x[y]`
    Index(CheckedPlaceExpr<'db>, CheckedExpr<'db>),

    /// an error has been reported
    Error(Reported),
}

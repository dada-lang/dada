use dada_ir_ast::{
    ast::{BinaryOp, Literal},
    diagnostic::Reported,
    span::Span,
};
use dada_ir_sym::ty::SymTy;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Expr<'chk, 'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,
    pub kind: &'chk ExprKind<'chk, 'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum ExprKind<'chk, 'db> {
    /// `$expr1; $expr2`
    Semi(Expr<'chk, 'db>, Expr<'chk, 'db>),

    /// `(...)`
    Tuple(Vec<Expr<'chk, 'db>>),

    /// `22`
    Literal(Literal<'db>),
}

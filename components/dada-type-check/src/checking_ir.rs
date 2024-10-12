use dada_ir_ast::{ast::Literal, diagnostic::Reported, span::Span};
use dada_ir_sym::{
    class::SymField,
    symbol::SymLocalVariable,
    ty::{SymPlace, SymPlaceKind, SymTy},
};

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

    /// `let $lv: $ty in $expr`
    LetIn(SymLocalVariable<'db>, SymTy<'db>, Expr<'chk, 'db>),

    /// `$place = $expr`
    Assign(PlaceExpr<'chk, 'db>, Expr<'chk, 'db>),

    /// `$place.give`
    Give(PlaceExpr<'chk, 'db>),

    /// `$place.lease`
    Lease(PlaceExpr<'chk, 'db>),

    /// `$place.share` or just `$place`
    Share(PlaceExpr<'chk, 'db>),

    /// Error occurred somewhere.
    Error(Reported),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct PlaceExpr<'chk, 'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,
    pub kind: &'chk PlaceExprKind<'chk, 'db>,
}

impl<'chk, 'db> PlaceExpr<'chk, 'db> {
    pub fn to_sym_place(&self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match self.kind {
            PlaceExprKind::Local(local) => SymPlace::new(db, SymPlaceKind::LocalVariable(*local)),
            PlaceExprKind::Field(place, field) => SymPlace::new(
                db,
                SymPlaceKind::Field(place.to_sym_place(db), field.name(db)),
            ),
            PlaceExprKind::Error(r) => SymPlace::new(db, SymPlaceKind::Error(*r)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum PlaceExprKind<'chk, 'db> {
    Local(SymLocalVariable<'db>),
    Field(PlaceExpr<'chk, 'db>, SymField<'db>),
    Error(Reported),
}

use dada_ir_ast::{ast::Literal, diagnostic::Reported, span::Span};
use dada_ir_sym::{
    class::SymField,
    symbol::SymLocalVariable,
    ty::{SymPlace, SymPlaceKind, SymTy},
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

    /// `let $lv: $ty [= $initializer] in $body`
    LetIn {
        lv: SymLocalVariable<'db>,
        ty: SymTy<'db>,
        initializer: Option<Expr<'chk, 'db>>,
        body: Expr<'chk, 'db>,
    },

    /// `$place = $expr`
    Assign {
        place: PlaceExpr<'chk, 'db>,
        expr: Expr<'chk, 'db>,
    },

    /// `$0.give`
    Give(PlaceExpr<'chk, 'db>),

    /// `$0.lease`
    Lease(PlaceExpr<'chk, 'db>),

    /// `$0.share` or just `$place`
    Share(PlaceExpr<'chk, 'db>),

    /// An expression that has not yet been computed
    /// because insufficient type information was
    /// available. The [`Check`](crate::executor::Check)
    /// stores an array, indexed by `DeferIndex`,
    /// which will eventually contain the expr to use here.
    Deferred(DeferIndex),

    /// Error occurred somewhere.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct PlaceExpr<'chk, 'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,
    pub kind: &'chk PlaceExprKind<'chk, 'db>,
}

impl<'chk, 'db> PlaceExpr<'chk, 'db> {
    pub fn to_sym_place(&self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match self.kind {
            PlaceExprKind::Local(local) => SymPlace::new(db, SymPlaceKind::Var(*local)),
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DeferIndex(usize);

impl From<usize> for DeferIndex {
    fn from(index: usize) -> Self {
        DeferIndex(index)
    }
}

impl DeferIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

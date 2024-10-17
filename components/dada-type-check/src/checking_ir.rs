use dada_ir_ast::{ast::Literal, diagnostic::Reported, span::Span};
use dada_ir_sym::{
    class::SymField,
    function::SymFunction,
    symbol::SymVariable,
    ty::{SymGenericTerm, SymPlace, SymPlaceKind, SymTy},
};

use crate::env::Env;

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
        lv: SymVariable<'db>,
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

    /// `$0[$1..]($2..)`
    ///
    /// During construction we ensure that the arities match and terms are well-kinded
    /// (or generate errors).
    Call(
        SymFunction<'db>,
        Vec<SymGenericTerm<'db>>,
        Vec<Expr<'chk, 'db>>,
    ),

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
    pub fn to_sym_place(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> SymPlace<'db> {
        match self.kind {
            PlaceExprKind::Var(local) => local.into_generic_term(db, &env.scope).assert_place(db),
            PlaceExprKind::Field(place, field) => SymPlace::new(
                db,
                SymPlaceKind::Field(place.to_sym_place(db, env), field.name(db)),
            ),
            PlaceExprKind::Error(r) => SymPlace::new(db, SymPlaceKind::Error(*r)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum PlaceExprKind<'chk, 'db> {
    Var(SymVariable<'db>),
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

use dada_ir_ast::{
    ast::{AstPath, BinaryOp, Literal, SpanVec, SpannedIdentifier},
    diagnostic::{Diagnostic, Level, Reported},
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymField,
    scope::{NameResolution, Resolve},
    symbol::SymLocalVariable,
    ty::{SymGenericArg, SymPlace, SymPlaceKind, SymTy},
    IntoSymInScope,
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
    /// `x`
    LocalVariable(SymLocalVariable<'db>),

    /// `x.f`
    Field(SymPlaceExpr<'db>, SpannedIdentifier<'db>),

    /// `x[y]`
    Index(SymPlaceExpr<'db>, SymExpr<'db>),

    /// an error has been reported
    Error(Reported),
}

impl<'db> SymPlaceExpr<'db> {
    /// Create a new place expression extended with a field `field`.
    pub fn field(self, db: &'db dyn crate::Db, field: SpannedIdentifier<'db>) -> Self {
        SymPlaceExpr::new(
            db,
            self.span(db).to(field.span),
            SymPlaceExprKind::Field(self, field),
        )
    }

    /// Convert from a "place expression", which is syntactic and tied to a particular span,
    /// to a "place", the symbolic, interned representation of a place.
    pub fn into_place(self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match self.kind(db) {
            SymPlaceExprKind::LocalVariable(x) => SymPlace::new(db, SymPlaceKind::LocalVariable(x)),
            SymPlaceExprKind::Field(x, f) => {
                SymPlace::new(db, SymPlaceKind::Field(x.into_place(db), f.id))
            }
            SymPlaceExprKind::Index(x, _y) => {
                SymPlace::new(db, SymPlaceKind::Index(x.into_place(db)))
            }
            SymPlaceExprKind::Error(r) => SymPlace::new(db, SymPlaceKind::Error(r)),
        }
    }
}

impl<'db> IntoSymInScope<'db> for AstPath<'db> {
    type Symbolic = SymPlaceExpr<'db>;

    fn into_sym_in_scope(
        self,
        db: &'db dyn crate::Db,
        scope: &crate::scope::Scope<'_, 'db>,
    ) -> Self::Symbolic {
        let (var, fields) = self.ids(db).split_first().unwrap();
        match var.resolve_in(db, scope) {
            Ok(r) => {
                todo!()
            }
            Err(r) => SymPlaceExpr::new(db, self.span(db), SymPlaceExprKind::Error(r)),
        }
    }
}

impl<'db> NameResolution<'db> {
    fn to_place_expr(
        self,
        db: &'db dyn crate::Db,
        source: &impl Spanned<'db>,
        ids: &'db [SpannedIdentifier<'db>],
    ) -> SymPlaceExpr<'db> {
        let (this, ids) = match self.resolve_relative(db, ids) {
            Ok((this, ids)) => (this, ids),
            Err(r) => return SymPlaceExpr::new(db, source.span(db), SymPlaceExprKind::Error(r)),
        };

        let NameResolution::SymLocalVariable(sym_local_variable) = this else {
            return SymPlaceExpr::new(
                db,
                source.span(db),
                SymPlaceExprKind::Error(
                    Diagnostic::error(
                        db,
                        source.span(db),
                        format!("expected place expression, found {}", this.categorize(db)),
                    )
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!(
                            "I expected a place expression, but I found {}",
                            this.describe(db)
                        ),
                    )
                    .report(db),
                ),
            );
        };

        let mut place_expr = SymPlaceExpr::new(
            db,
            source.span(db),
            SymPlaceExprKind::LocalVariable(sym_local_variable),
        );

        for &id in ids {
            place_expr = place_expr.field(db, id);
        }

        place_expr
    }
}

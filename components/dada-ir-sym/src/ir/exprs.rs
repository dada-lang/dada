//! The "object IR" is an intermediate IR that we create
//! as a first pass for type checking. The name "object"
//! derives from the fact that it doesn't track precise
//! types, but rather just the type of the underlying
//! object without any permissions (i.e., what class/struct/enum/etc is it?).
//! This can then be used to bootstrap full type checking.
//!
//! We need to create this IR first because full type checking will
//! require knowing which variables are live. Knowing that
//! requires that we have fully parsed the AST. But fully parsing
//! the AST requires being able to disambiguate things like `x.foo[..]()`,
//! which could be either indexing a field `foo` and then calling the
//! result or invoking a method `foo` with generic arguments.
//! The object IR gives us enough information to make those determinations.

use crate::{
    ir::class::SymField,
    ir::functions::SymFunction,
    ir::variables::{FromVar, SymVariable},
    ir::types::{SymGenericTerm, SymPlace, SymTy},
};
use dada_ir_ast::{
    ast::{AstBinaryOp, PermissionOp},
    diagnostic::{Err, Reported},
    span::Span,
};
use ordered_float::OrderedFloat;
use salsa::Update;

use crate::exprs::Temporary;

#[salsa::tracked]
pub struct SymExpr<'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,

    #[return_ref]
    pub kind: SymExprKind<'db>,
}

impl<'db> Err<'db> for SymExpr<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, r: Reported) -> Self {
        SymExpr::new(db, r.span(db), SymTy::err(db, r), SymExprKind::Error(r))
    }
}

impl<'db> SymExpr<'db> {
    pub(crate) fn into_temporary(
        self,
        db: &'db dyn crate::Db,
        temporaries: &mut Vec<Temporary<'db>>,
    ) -> SymPlaceExpr<'db> {
        let ty = self.ty(db);

        // Create a temporary to store the result of this expression.
        let temporary = Temporary::new(db, self.span(db), self.ty(db), Some(self));
        let lv = temporary.lv;
        temporaries.push(temporary);

        // The result will be a reference to that temporary.
        SymPlaceExpr::new(db, self.span(db), ty, SymPlaceExprKind::Var(lv))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum SymExprKind<'db> {
    /// `$expr1; $expr2`
    Semi(SymExpr<'db>, SymExpr<'db>),

    /// `(...)`
    Tuple(Vec<SymExpr<'db>>),

    /// `22` etc
    Primitive(SymLiteral),

    /// `let $lv: $ty [= $initializer] in $body`
    LetIn {
        lv: SymVariable<'db>,
        ty: SymTy<'db>,
        initializer: Option<SymExpr<'db>>,
        body: SymExpr<'db>,
    },

    /// `future.await`
    Await {
        future: SymExpr<'db>,
        await_keyword: Span<'db>,
    },

    /// `$place = $expr`
    Assign {
        place: SymPlaceExpr<'db>,
        value: SymExpr<'db>,
    },

    /// `$0.lease` etc
    PermissionOp(PermissionOp, SymPlaceExpr<'db>),

    /// `$0[$1..]($2..)`
    ///
    /// During construction we ensure that the arities match and terms are well-kinded
    /// (or generate errors).
    Call {
        function: SymFunction<'db>,
        substitution: Vec<SymGenericTerm<'db>>,
        arg_temps: Vec<SymVariable<'db>>,
    },

    /// Return a value from this function
    Return(SymExpr<'db>),

    /// Boolean not
    Not {
        operand: SymExpr<'db>,
        op_span: Span<'db>,
    },

    /// `a + b` etc
    BinaryOp(SymBinaryOp, SymExpr<'db>, SymExpr<'db>),

    /// Something like `Point { x: ..., y: ... }`
    Aggregate {
        ty: SymTy<'db>,
        fields: Vec<SymExpr<'db>>,
    },

    /// Match, if/else-if chain, etc
    Match { arms: Vec<SymMatchArm<'db>> },

    /// Error occurred somewhere.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum SymLiteral {
    /// Have to check the type of the expression to determine how to interpret these bits
    Integral { bits: u64 },

    /// Have to check the type of the expression to determine how to interpret these bits
    Float { bits: OrderedFloat<f64> },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    EqualEqual,
}

impl TryFrom<AstBinaryOp> for SymBinaryOp {
    type Error = dada_util::Error;

    fn try_from(value: AstBinaryOp) -> Result<Self, Self::Error> {
        match value {
            AstBinaryOp::Add => Ok(SymBinaryOp::Add),
            AstBinaryOp::Sub => Ok(SymBinaryOp::Sub),
            AstBinaryOp::Mul => Ok(SymBinaryOp::Mul),
            AstBinaryOp::Div => Ok(SymBinaryOp::Div),
            AstBinaryOp::GreaterThan => Ok(SymBinaryOp::GreaterThan),
            AstBinaryOp::LessThan => Ok(SymBinaryOp::LessThan),
            AstBinaryOp::GreaterEqual => Ok(SymBinaryOp::GreaterEqual),
            AstBinaryOp::LessEqual => Ok(SymBinaryOp::LessEqual),
            AstBinaryOp::EqualEqual => Ok(SymBinaryOp::EqualEqual),
            AstBinaryOp::AndAnd | AstBinaryOp::OrOr | AstBinaryOp::Assign => {
                dada_util::bail!("no equivalent object binary op")
            }
        }
    }
}

/// A match arm is one part of a match statement.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub struct SymMatchArm<'db> {
    // FIXME: patterns
    /// Condition to evaluate; if `None` then it always applies
    pub condition: Option<SymExpr<'db>>,

    /// Body to evaluate.
    pub body: SymExpr<'db>,
}

#[salsa::tracked]
pub struct SymPlaceExpr<'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,

    #[return_ref]
    pub kind: SymPlaceExprKind<'db>,
}

impl<'db> Err<'db> for SymPlaceExpr<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, r: Reported) -> Self {
        SymPlaceExpr::new(
            db,
            r.span(db),
            SymTy::err(db, r),
            SymPlaceExprKind::Error(r),
        )
    }
}

impl<'db> SymPlaceExpr<'db> {
    pub fn give(self, db: &'db dyn crate::Db) -> SymExpr<'db> {
        SymExpr::new(
            db,
            self.span(db),
            self.ty(db),
            SymExprKind::PermissionOp(PermissionOp::Give, self),
        )
    }

    pub fn into_sym_place(self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match *self.kind(db) {
            SymPlaceExprKind::Var(lv) => SymPlace::var(db, lv),
            SymPlaceExprKind::Field(place, field) => place.into_sym_place(db).field(db, field),
            SymPlaceExprKind::Error(r) => SymPlace::err(db, r),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum SymPlaceExprKind<'db> {
    Var(SymVariable<'db>),
    Field(SymPlaceExpr<'db>, SymField<'db>),
    Error(Reported),
}

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
    class::SymField,
    function::SymFunction,
    symbol::{FromVar, SymVariable},
    ty::{SymGenericTerm, SymPlace, SymTy},
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
pub struct ObjectExpr<'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,

    #[return_ref]
    pub kind: ObjectExprKind<'db>,
}

impl<'db> Err<'db> for ObjectExpr<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, r: Reported) -> Self {
        ObjectExpr::new(db, r.span(db), SymTy::err(db, r), ObjectExprKind::Error(r))
    }
}

impl<'db> ObjectExpr<'db> {
    pub(crate) fn into_temporary(
        self,
        db: &'db dyn crate::Db,
        temporaries: &mut Vec<Temporary<'db>>,
    ) -> ObjectPlaceExpr<'db> {
        let ty = self.ty(db);

        // Create a temporary to store the result of this expression.
        let temporary = Temporary::new(db, self.span(db), self.ty(db), Some(self));
        let lv = temporary.lv;
        temporaries.push(temporary);

        // The result will be a reference to that temporary.
        ObjectPlaceExpr::new(db, self.span(db), ty, ObjectPlaceExprKind::Var(lv))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum ObjectExprKind<'db> {
    /// `$expr1; $expr2`
    Semi(ObjectExpr<'db>, ObjectExpr<'db>),

    /// `(...)`
    Tuple(Vec<ObjectExpr<'db>>),

    /// `22` etc
    Primitive(PrimitiveLiteral),

    /// `let $lv: $ty [= $initializer] in $body`
    LetIn {
        lv: SymVariable<'db>,
        ty: SymTy<'db>,
        initializer: Option<ObjectExpr<'db>>,
        body: ObjectExpr<'db>,
    },

    /// `future.await`
    Await {
        future: ObjectExpr<'db>,
        await_keyword: Span<'db>,
    },

    /// `$place = $expr`
    Assign {
        place: ObjectPlaceExpr<'db>,
        value: ObjectExpr<'db>,
    },

    /// `$0.lease` etc
    PermissionOp(PermissionOp, ObjectPlaceExpr<'db>),

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
    Return(ObjectExpr<'db>),

    /// Boolean not
    Not {
        operand: ObjectExpr<'db>,
        op_span: Span<'db>,
    },

    /// `a + b` etc
    BinaryOp(ObjectBinaryOp, ObjectExpr<'db>, ObjectExpr<'db>),

    /// Something like `Point { x: ..., y: ... }`
    Aggregate {
        ty: SymTy<'db>,
        fields: Vec<ObjectExpr<'db>>,
    },

    /// Match, if/else-if chain, etc
    Match { arms: Vec<MatchArm<'db>> },

    /// Error occurred somewhere.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum PrimitiveLiteral {
    /// Have to check the type of the expression to determine how to interpret these bits
    Integral { bits: u64 },

    /// Have to check the type of the expression to determine how to interpret these bits
    Float { bits: OrderedFloat<f64> },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum ObjectBinaryOp {
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

impl TryFrom<AstBinaryOp> for ObjectBinaryOp {
    type Error = dada_util::Error;

    fn try_from(value: AstBinaryOp) -> Result<Self, Self::Error> {
        match value {
            AstBinaryOp::Add => Ok(ObjectBinaryOp::Add),
            AstBinaryOp::Sub => Ok(ObjectBinaryOp::Sub),
            AstBinaryOp::Mul => Ok(ObjectBinaryOp::Mul),
            AstBinaryOp::Div => Ok(ObjectBinaryOp::Div),
            AstBinaryOp::GreaterThan => Ok(ObjectBinaryOp::GreaterThan),
            AstBinaryOp::LessThan => Ok(ObjectBinaryOp::LessThan),
            AstBinaryOp::GreaterEqual => Ok(ObjectBinaryOp::GreaterEqual),
            AstBinaryOp::LessEqual => Ok(ObjectBinaryOp::LessEqual),
            AstBinaryOp::EqualEqual => Ok(ObjectBinaryOp::EqualEqual),
            AstBinaryOp::AndAnd | AstBinaryOp::OrOr | AstBinaryOp::Assign => {
                dada_util::bail!("no equivalent object binary op")
            }
        }
    }
}

/// A match arm is one part of a match statement.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub struct MatchArm<'db> {
    // FIXME: patterns
    /// Condition to evaluate; if `None` then it always applies
    pub condition: Option<ObjectExpr<'db>>,

    /// Body to evaluate.
    pub body: ObjectExpr<'db>,
}

#[salsa::tracked]
pub struct ObjectPlaceExpr<'db> {
    pub span: Span<'db>,
    pub ty: SymTy<'db>,

    #[return_ref]
    pub kind: ObjectPlaceExprKind<'db>,
}

impl<'db> Err<'db> for ObjectPlaceExpr<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, r: Reported) -> Self {
        ObjectPlaceExpr::new(
            db,
            r.span(db),
            SymTy::err(db, r),
            ObjectPlaceExprKind::Error(r),
        )
    }
}

impl<'db> ObjectPlaceExpr<'db> {
    pub fn give(self, db: &'db dyn crate::Db) -> ObjectExpr<'db> {
        ObjectExpr::new(
            db,
            self.span(db),
            self.ty(db),
            ObjectExprKind::PermissionOp(PermissionOp::Give, self),
        )
    }

    pub fn into_sym_place(self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match *self.kind(db) {
            ObjectPlaceExprKind::Var(lv) => SymPlace::var(db, lv),
            ObjectPlaceExprKind::Field(place, field) => place.into_sym_place(db).field(db, field),
            ObjectPlaceExprKind::Error(r) => SymPlace::err(db, r),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum ObjectPlaceExprKind<'db> {
    Var(SymVariable<'db>),
    Field(ObjectPlaceExpr<'db>, SymField<'db>),
    Error(Reported),
}

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

use dada_ir_ast::{
    ast::Literal,
    diagnostic::{Err, Reported},
    span::Span,
};
use dada_ir_sym::{
    class::SymField,
    function::SymFunction,
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{FromVar, SymGenericTerm, SymTy, SymTyName, Var},
};
use dada_util::FromImpls;
use salsa::Update;

#[salsa::tracked]
pub struct ObjectExpr<'db> {
    pub span: Span<'db>,
    pub ty: ObjectTy<'db>,

    #[return_ref]
    pub kind: ObjectExprKind<'db>,
}

impl<'db> Err<'db> for ObjectExpr<'db> {
    fn err(db: &'db dyn salsa::Database, r: Reported) -> Self {
        ObjectExpr::new(
            db,
            r.span(db),
            ObjectTy::err(db, r),
            ObjectExprKind::Error(r),
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum ObjectExprKind<'db> {
    /// `$expr1; $expr2`
    Semi(ObjectExpr<'db>, ObjectExpr<'db>),

    /// `(...)`
    Tuple(Vec<ObjectExpr<'db>>),

    /// `22`
    Literal(Literal<'db>),

    /// `let $lv: $ty [= $initializer] in $body`
    LetIn {
        lv: SymVariable<'db>,

        // If this is a true local variable (as opposed to a temporary),
        // then this will be its "sym ty". For temporaries, it's just None
        // because no sym ty has been created yet.
        sym_ty: Option<SymTy<'db>>,

        ty: ObjectTy<'db>,
        initializer: Option<ObjectExpr<'db>>,
        body: ObjectExpr<'db>,
    },

    /// `$place = $expr`
    Assign {
        place: ObjectPlaceExpr<'db>,
        expr: ObjectExpr<'db>,
    },

    /// `$0.give`
    Give(ObjectPlaceExpr<'db>),

    /// `$0.lease`
    Lease(ObjectPlaceExpr<'db>),

    /// `$0.share` or just `$place`
    Share(ObjectPlaceExpr<'db>),

    /// `$0[$1..]($2..)`
    ///
    /// During construction we ensure that the arities match and terms are well-kinded
    /// (or generate errors).
    Call {
        function: SymFunction<'db>,
        class_substitution: Vec<SymGenericTerm<'db>>,
        method_substitution: Vec<SymGenericTerm<'db>>,
        arg_temps: Vec<SymVariable<'db>>,
    },

    /// Return a value from this function
    Return(ObjectExpr<'db>),

    /// Error occurred somewhere.
    Error(Reported),
}

#[salsa::tracked]
pub struct ObjectPlaceExpr<'db> {
    pub span: Span<'db>,
    pub ty: ObjectTy<'db>,

    #[return_ref]
    pub kind: ObjectPlaceExprKind<'db>,
}

impl<'db> Err<'db> for ObjectPlaceExpr<'db> {
    fn err(db: &'db dyn salsa::Database, r: Reported) -> Self {
        ObjectPlaceExpr::new(
            db,
            r.span(db),
            ObjectTy::err(db, r),
            ObjectPlaceExprKind::Error(r),
        )
    }
}

impl<'db> ObjectPlaceExpr<'db> {
    pub fn to_object_place(&self) -> ObjectGenericTerm<'db> {
        ObjectGenericTerm::Place
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Update)]
pub enum ObjectPlaceExprKind<'db> {
    Var(SymVariable<'db>),
    Field(ObjectPlaceExpr<'db>, SymField<'db>),
    Error(Reported),
}

#[salsa::interned]
pub struct ObjectTy<'db> {
    #[return_ref]
    pub kind: ObjectTyKind<'db>,
}

impl<'db> ObjectTy<'db> {
    pub fn unit(db: &'db dyn crate::Db) -> ObjectTy<'db> {
        SymTy::unit(db).into_object_ir(db)
    }

    pub fn shared(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }

    pub fn never(db: &'db dyn crate::Db) -> ObjectTy<'db> {
        SymTy::never(db).into_object_ir(db)
    }
}

impl<'db> Err<'db> for ObjectTy<'db> {
    fn err(db: &'db dyn salsa::Database, r: Reported) -> Self {
        ObjectTy::new(db, ObjectTyKind::Error(r))
    }
}

impl<'db> HasKind<'db> for ObjectTy<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Type
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum ObjectTyKind<'db> {
    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    ///
    /// Important: the generic arguments must be well-kinded and of the correct number.
    Named(SymTyName<'db>, Vec<ObjectGenericTerm<'db>>),

    /// Reference to a generic or inference variable, e.g., `T` or `?X`
    Var(Var<'db>),

    /// Indicates a value that can never be created, denoted `!`.
    Never,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

/// Value of a generic parameter
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum ObjectGenericTerm<'db> {
    Type(ObjectTy<'db>),
    #[no_from_impl]
    Perm,
    #[no_from_impl]
    Place,
    Error(Reported),
}

impl<'db> HasKind<'db> for ObjectGenericTerm<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        match self {
            ObjectGenericTerm::Type(_) => kind == SymGenericKind::Type,
            ObjectGenericTerm::Perm => kind == SymGenericKind::Perm,
            ObjectGenericTerm::Place => kind == SymGenericKind::Place,
            ObjectGenericTerm::Error(Reported(_)) => true,
        }
    }
}

impl<'db> FromVar<'db> for ObjectGenericTerm<'db> {
    fn var(db: &'db dyn crate::Db, kind: SymGenericKind, var: Var<'db>) -> Self {
        SymGenericTerm::var(db, kind, var).into_object_ir(db)
    }
}

impl<'db> ObjectGenericTerm<'db> {
    pub fn assert_type(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        match self {
            ObjectGenericTerm::Type(ty) => ty,
            ObjectGenericTerm::Error(r) => ObjectTy::new(db, ObjectTyKind::Error(r)),
            _ => panic!("`{self:?}` is not a type"),
        }
    }
}

pub(crate) trait IntoObjectIr<'db>: Update {
    type Object: Update;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> Self::Object;
}

mod into_object_ir_impls;
mod subst_impls;

use dada_ir_ast::{ast::Literal, diagnostic::Reported, span::Span};
use dada_ir_sym::{
    class::SymField,
    function::SymFunction,
    symbol::{HasKind, SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymTy, SymTyName, Var},
};
use dada_util::FromImpls;
use salsa::Update;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Expr<'chk, 'db> {
    pub span: Span<'db>,
    pub ty: ObjectTy<'db>,
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
        ty: ObjectTy<'db>,
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
    Call {
        function: SymFunction<'db>,
        class_substitution: Vec<SymGenericTerm<'db>>,
        method_substitution: Vec<SymGenericTerm<'db>>,
        arg_temps: Vec<SymVariable<'db>>,
    },

    /// Error occurred somewhere.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct PlaceExpr<'chk, 'db> {
    pub span: Span<'db>,
    pub ty: ObjectTy<'db>,
    pub kind: &'chk PlaceExprKind<'chk, 'db>,
}

impl<'chk, 'db> PlaceExpr<'chk, 'db> {
    pub fn to_object_place(&self) -> ObjectGenericTerm<'db> {
        ObjectGenericTerm::Place
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum PlaceExprKind<'chk, 'db> {
    Var(SymVariable<'db>),
    Field(PlaceExpr<'chk, 'db>, SymField<'db>),
    Error(Reported),
}

#[salsa::interned]
pub(crate) struct ObjectTy<'db> {
    #[return_ref]
    pub kind: ObjectTyKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub(crate) enum ObjectTyKind<'db> {
    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    ///
    /// Important: the generic arguments must be well-kinded and of the correct number.
    Named(SymTyName<'db>, Vec<ObjectGenericTerm<'db>>),

    /// Reference to a generic or inference variable, e.g., `T` or `?X`
    Var(Var<'db>),

    /// Indicates the user wrote `?` and we should use gradual typing.
    Unknown,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

impl<'db> ObjectTy<'db> {
    pub fn unit(db: &'db dyn crate::Db) -> ObjectTy<'db> {
        SymTy::unit(db).into_object_ir(db)
    }

    pub fn error(db: &'db dyn crate::Db, reported: Reported) -> ObjectTy<'db> {
        ObjectTy::new(db, ObjectTyKind::Error(reported))
    }

    pub fn shared(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }
}

/// Value of a generic parameter
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub(crate) enum ObjectGenericTerm<'db> {
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
            ObjectGenericTerm::Error(Reported) => true,
        }
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

pub trait IntoObjectIr<'db>: Update {
    type Object: Update;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> Self::Object;
}

mod subst_impls;
mod to_object_impls;

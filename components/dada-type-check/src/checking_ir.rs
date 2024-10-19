use dada_ir_ast::{ast::Literal, diagnostic::Reported, span::Span};
use dada_ir_sym::{
    class::SymField,
    function::SymFunction,
    symbol::{SymGenericKind, SymVariable},
    ty::{Binder, SymGenericTerm, SymPlace, SymPlaceKind, SymTy, SymTyKind, SymTyName, Var},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::env::Env;

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
        SymTy::unit(db).to_object(db)
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

impl<'db> ObjectGenericTerm<'db> {
    pub fn has_kind(self, kind: SymGenericKind) -> bool {
        match self {
            ObjectGenericTerm::Type(_) => kind == SymGenericKind::Type,
            ObjectGenericTerm::Perm => kind == SymGenericKind::Perm,
            ObjectGenericTerm::Place => kind == SymGenericKind::Place,
            ObjectGenericTerm::Error(Reported) => true,
        }
    }

    pub fn assert_type(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        match self {
            ObjectGenericTerm::Type(ty) => ty,
            ObjectGenericTerm::Error(r) => ObjectTy::new(db, ObjectTyKind::Error(r)),
            _ => panic!("`{self:?}` is not a type"),
        }
    }
}

pub trait ToObject<'db>: Update {
    type Object: Update;

    fn to_object(&self, db: &'db dyn crate::Db) -> Self::Object;
}

impl<'db> ToObject<'db> for ObjectTy<'db> {
    type Object = Self;

    fn to_object(&self, _db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }
}

impl<'db> ToObject<'db> for SymTy<'db> {
    type Object = ObjectTy<'db>;

    fn to_object(&self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        match self.kind(db) {
            SymTyKind::Perm(_, ty) => ty.to_object(db),
            SymTyKind::Named(name, vec) => ObjectTy::new(
                db,
                ObjectTyKind::Named(*name, vec.iter().map(|t| t.to_object(db)).collect()),
            ),
            SymTyKind::Var(var) => ObjectTy::new(db, ObjectTyKind::Var(*var)),
            SymTyKind::Unknown => ObjectTy::new(db, ObjectTyKind::Unknown),
            SymTyKind::Error(reported) => ObjectTy::new(db, ObjectTyKind::Error(*reported)),
        }
    }
}

impl<'db> ToObject<'db> for SymGenericTerm<'db> {
    type Object = ObjectGenericTerm<'db>;

    fn to_object(&self, db: &'db dyn crate::Db) -> ObjectGenericTerm<'db> {
        match self {
            SymGenericTerm::Type(ty) => ObjectGenericTerm::Type(ty.to_object(db)),
            SymGenericTerm::Perm(_) => ObjectGenericTerm::Perm,
            SymGenericTerm::Error(reported) => ObjectGenericTerm::Error(*reported),
            SymGenericTerm::Place(_) => ObjectGenericTerm::Place,
        }
    }
}

impl<'db, T> ToObject<'db> for Binder<T>
where
    T: ToObject<'db>,
{
    type Object = Binder<T::Object>;

    fn to_object(&self, db: &'db dyn crate::Db) -> Self::Object {
        self.map(db, (), |db, (), t| t.to_object(db))
    }
}

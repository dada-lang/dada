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
    ast::{Literal, SpannedBinaryOp},
    diagnostic::{Err, Reported},
    span::Span,
};
use dada_ir_sym::{
    binder::LeafBoundTerm,
    class::SymField,
    function::SymFunction,
    indices::InferVarIndex,
    symbol::{FromVar, HasKind, SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymTy, SymTyName},
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

    /// `future.await`
    Await {
        future: ObjectExpr<'db>,
        await_keyword: Span<'db>,
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
        substitution: Vec<SymGenericTerm<'db>>,
        arg_temps: Vec<SymVariable<'db>>,
    },

    /// Return a value from this function
    Return(ObjectExpr<'db>),

    /// `a + b` etc
    BinaryOp(SpannedBinaryOp<'db>, ObjectExpr<'db>, ObjectExpr<'db>),

    /// Something like `Point { x: ..., y: ... }`
    Aggregate {
        ty: ObjectTy<'db>,
        fields: Vec<ObjectExpr<'db>>,
    },

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

    pub fn give(self, db: &'db dyn crate::Db) -> ObjectExpr<'db> {
        ObjectExpr::new(db, self.span(db), self.ty(db), ObjectExprKind::Give(self))
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

    pub fn shared(self, _db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }

    pub fn never(db: &'db dyn crate::Db) -> ObjectTy<'db> {
        SymTy::never(db).into_object_ir(db)
    }
}

impl<'db> LeafBoundTerm<'db> for ObjectTy<'db> {}

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

impl<'db> FromVar<'db> for ObjectTy<'db> {
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        assert_eq!(var.kind(db), SymGenericKind::Type);
        ObjectTy::new(db, ObjectTyKind::Var(var))
    }
}

impl std::fmt::Display for ObjectTy<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| match self.kind(db) {
            ObjectTyKind::Named(name, vec) => {
                write!(f, "{name}")?;

                if !vec.is_empty() {
                    write!(f, "[")?;
                    for (arg, index) in vec.iter().zip(0..) {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                    }
                    write!(f, "]")?;
                }

                Ok(())
            }
            ObjectTyKind::Var(var) => match var.name(db) {
                Some(name) => write!(f, "{name}"),
                None => write!(f, "/* some {} */", var.kind(db)),
            },
            ObjectTyKind::Infer(var) => write!(f, "/* ?{} */", var.as_usize()),
            ObjectTyKind::Never => write!(f, "!"),
            ObjectTyKind::Error(_) => write!(f, "/* error */"),
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum ObjectTyKind<'db> {
    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    ///
    /// Important: the generic arguments must be well-kinded and of the correct number.
    Named(SymTyName<'db>, Vec<ObjectGenericTerm<'db>>),

    /// Reference to a generic, e.g., `T`.
    Var(SymVariable<'db>),

    /// Inference variable, e.g., `?X`.
    Infer(InferVarIndex),

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

impl std::fmt::Display for ObjectGenericTerm<'_> {
    fn fmt(&self, _fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
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
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        SymGenericTerm::var(db, var).into_object_ir(db)
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

    /// A term is an "isolated" bound if it has no super- or sub-terms.
    /// For example, `u32` is an isolated type because it is not a super- or sub-type of anything else.
    ///
    /// This is used to improve type inference: an isolated bound propagates from lower- to upper.
    ///
    /// The term "isolated" comes from lattice theory: *An "isolated element lattice" refers to a lattice
    /// structure where a specific element within the lattice is positioned in a way that it has no direct
    /// neighbors or connections to other elements within the same lattice, essentially being "isolated"
    /// from the surrounding structure.*
    pub fn is_isolated(self, db: &'db dyn crate::Db) -> bool {
        match self {
            ObjectGenericTerm::Type(object_ty) => match object_ty.kind(db) {
                ObjectTyKind::Named(name, vec) => match name {
                    SymTyName::Class(_class) => {
                        // FIXME: This will be true for some classes but not others
                        false
                    }

                    SymTyName::Primitive(_) | SymTyName::Tuple { arity: _ } | SymTyName::Future => {
                        vec.iter().all(|arg| arg.is_isolated(db))
                    }
                },

                // We treat universal variables as *if* they are isolated,
                // but we don't actually know what type they represent, so they
                // are not truly isolated.
                // Returning false here preserves our ability to add subtyping bounds
                // in the future, for example.
                ObjectTyKind::Var(_) => false,

                // Inference variables have growing and unknown set of bounds, not isolated.
                ObjectTyKind::Infer(_) => false,
                ObjectTyKind::Never => true,
                ObjectTyKind::Error(_) => true,
            },

            // Just one element.
            ObjectGenericTerm::Perm => true,

            // Just one element.
            ObjectGenericTerm::Place => true,

            // We call errors "isolated" because they can (harmlessly) propagate around.
            ObjectGenericTerm::Error(_) => true,
        }
    }
}

pub(crate) trait IntoObjectIr<'db>: Update {
    type Object: Update;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> Self::Object;
}

mod into_object_ir_impls;
mod subst_impls;

use crate::{
    binder::LeafBoundTerm,
    class::{SymAggregate, SymField},
    env::EnvLike,
    indices::{FromInfer, FromInferVar, InferVarIndex},
    primitive::{SymPrimitive, SymPrimitiveKind},
    symbol::{AssertKind, FromVar, HasKind, SymGenericKind, SymVariable},
    Db, IntoSymbol, SymbolizeInEnv,
};
use dada_ir_ast::{
    ast::{AstGenericDecl, AstGenericKind, AstPerm, AstPermKind},
    diagnostic::{Err, Errors, Reported},
    span::Spanned,
};
use dada_util::FromImpls;
use salsa::Update;

/// Value of a generic parameter
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymGenericTerm<'db> {
    Type(SymTy<'db>),
    Perm(SymPerm<'db>),
    Place(SymPlace<'db>),
    Error(Reported),
}

impl<'db> std::fmt::Display for SymGenericTerm<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymGenericTerm::Type(ty) => write!(f, "{ty}"),
            SymGenericTerm::Perm(perm) => write!(f, "{perm}"),
            SymGenericTerm::Place(place) => write!(f, "{place}"),
            SymGenericTerm::Error(_) => write!(f, "<error>"),
        }
    }
}

impl<'db> HasKind<'db> for SymGenericTerm<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        match self {
            SymGenericTerm::Type(_) => kind == SymGenericKind::Type,
            SymGenericTerm::Perm(_) => kind == SymGenericKind::Perm,
            SymGenericTerm::Place(_) => kind == SymGenericKind::Place,
            SymGenericTerm::Error(Reported(_)) => true,
        }
    }
}

impl<'db> AssertKind<'db, SymTy<'db>> for SymGenericTerm<'db> {
    fn assert_kind(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        assert!(self.has_kind(db, SymGenericKind::Type));
        match self {
            SymGenericTerm::Type(v) => v,
            SymGenericTerm::Error(r) => SymTy::err(db, r),
            _ => unreachable!(),
        }
    }
}

impl<'db> AssertKind<'db, SymPerm<'db>> for SymGenericTerm<'db> {
    fn assert_kind(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        assert!(self.has_kind(db, SymGenericKind::Perm));
        match self {
            SymGenericTerm::Perm(v) => v,
            SymGenericTerm::Error(r) => SymPerm::err(db, r),
            _ => unreachable!(),
        }
    }
}

impl<'db> AssertKind<'db, SymPlace<'db>> for SymGenericTerm<'db> {
    fn assert_kind(self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        assert!(self.has_kind(db, SymGenericKind::Place));
        match self {
            SymGenericTerm::Place(v) => v,
            SymGenericTerm::Error(r) => SymPlace::err(db, r),
            _ => unreachable!(),
        }
    }
}

impl<'db> FromVar<'db> for SymGenericTerm<'db> {
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        match var.kind(db) {
            SymGenericKind::Type => SymTy::var(db, var).into(),
            SymGenericKind::Perm => SymPerm::var(db, var).into(),
            SymGenericKind::Place => SymPlace::var(db, var).into(),
        }
    }
}

impl<'db> FromInferVar<'db> for SymGenericTerm<'db> {
    fn infer(db: &'db dyn crate::Db, kind: SymGenericKind, index: InferVarIndex) -> Self {
        match kind {
            SymGenericKind::Type => SymTy::new(db, SymTyKind::Infer(index)).into(),
            SymGenericKind::Perm => SymPerm::new(db, SymPermKind::Infer(index)).into(),
            SymGenericKind::Place => SymPlace::new(db, SymPlaceKind::Infer(index)).into(),
        }
    }
}

impl<'db> SymGenericTerm<'db> {
    #[track_caller]
    pub fn assert_type(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        match self {
            SymGenericTerm::Type(ty) => ty,
            SymGenericTerm::Error(reported) => SymTy::new(db, SymTyKind::Error(reported)),
            _ => unreachable!(),
        }
    }

    #[track_caller]
    pub fn assert_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        match self {
            SymGenericTerm::Perm(perm) => perm,
            SymGenericTerm::Error(reported) => SymPerm::new(db, SymPermKind::Error(reported)),
            _ => unreachable!(),
        }
    }

    #[track_caller]
    pub fn assert_place(self, db: &'db dyn crate::Db) -> SymPlace<'db> {
        match self {
            SymGenericTerm::Place(place) => place,
            SymGenericTerm::Error(reported) => SymPlace::new(db, SymPlaceKind::Error(reported)),
            _ => unreachable!(),
        }
    }

    /// Returns the kind of term (or `Err` if it is an error).
    pub fn kind(self) -> Errors<SymGenericKind> {
        match self {
            SymGenericTerm::Type(_) => Ok(SymGenericKind::Type),
            SymGenericTerm::Perm(_) => Ok(SymGenericKind::Perm),
            SymGenericTerm::Place(_) => Ok(SymGenericKind::Place),
            SymGenericTerm::Error(r) => Err(r),
        }
    }

    /// True if self is an error or if it has the given kind.
    pub fn has_kind(self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        match self {
            SymGenericTerm::Type(_) => kind == SymGenericKind::Type,
            SymGenericTerm::Perm(_) => kind == SymGenericKind::Perm,
            SymGenericTerm::Place(_) => kind == SymGenericKind::Place,
            SymGenericTerm::Error(_) => true,
        }
    }
}

#[salsa::interned]
pub struct SymTy<'db> {
    #[return_ref]
    pub kind: SymTyKind<'db>,
}

impl<'db> SymTy<'db> {
    /// Returns a new [`SymTyKind::Named`][].
    pub fn named(
        db: &'db dyn Db,
        name: SymTyName<'db>,
        generics: Vec<SymGenericTerm<'db>>,
    ) -> Self {
        SymTy::new(db, SymTyKind::Named(name, generics))
    }

    /// Returns a [`SymTyKind::Named`][] type for `()`.
    pub fn unit(db: &'db dyn Db) -> Self {
        #[salsa::tracked]
        fn unit_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
            SymTy::named(db, SymTyName::Tuple { arity: 0 }, vec![])
        }

        unit_ty(db)
    }

    /// Returns a [`SymTyKind::Named`][] type for `bool`.
    pub fn boolean(db: &'db dyn Db) -> Self {
        SymTy::named(db, SymPrimitiveKind::Bool.intern(db).into(), vec![])
    }

    /// Returns a [`SymTyKind::Never`][] type.
    pub fn never(db: &'db dyn Db) -> Self {
        #[salsa::tracked]
        fn never_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
            SymTy::new(db, SymTyKind::Never)
        }

        never_ty(db)
    }

    /// Returns a new [`SymTyKind::Perm`][].
    pub fn perm(db: &'db dyn Db, perm: SymPerm<'db>, ty: SymTy<'db>) -> Self {
        SymTy::new(db, SymTyKind::Perm(perm, ty))
    }

    /// Returns a version of this type shared from `place`.
    pub fn shared(self, db: &'db dyn Db, place: SymPlace<'db>) -> Self {
        SymTy::new(
            db,
            SymTyKind::Perm(SymPerm::new(db, SymPermKind::Shared(vec![place])), self),
        )
    }

    /// Returns a version of this type leased from `place`.
    pub fn leased(self, db: &'db dyn Db, place: SymPlace<'db>) -> Self {
        SymTy::new(
            db,
            SymTyKind::Perm(SymPerm::new(db, SymPermKind::Leased(vec![place])), self),
        )
    }

    /// Returns a version of this type given from `place`.
    pub fn given(self, db: &'db dyn Db, place: SymPlace<'db>) -> Self {
        SymTy::new(
            db,
            SymTyKind::Perm(SymPerm::new(db, SymPermKind::Given(vec![place])), self),
        )
    }
}

impl<'db> FromInfer<'db> for SymTy<'db> {
    fn infer(db: &'db dyn crate::Db, var: InferVarIndex) -> Self {
        SymTy::new(db, SymTyKind::Infer(var))
    }
}

impl std::fmt::Display for SymTy<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| match self.kind(db) {
            SymTyKind::Named(name, generics) => {
                if generics.is_empty() {
                    write!(f, "{name}")
                } else {
                    write!(
                        f,
                        "{name}[{}]",
                        generics
                            .iter()
                            .map(|g| g.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            _ => write!(f, "{:?}", self.kind(db)),
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

impl<'db> SymbolizeInEnv<'db> for SymTy<'db> {
    type Output = SymTy<'db>;

    fn symbolize_in_env(&self, _env: &mut dyn EnvLike<'db>) -> Self::Output {
        *self
    }
}

impl<'db> LeafBoundTerm<'db> for SymTy<'db> {}

impl<'db> Err<'db> for SymTy<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        SymTy::new(db, SymTyKind::Error(reported))
    }
}

impl<'db> HasKind<'db> for SymTy<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Type
    }
}

impl<'db> FromVar<'db> for SymTy<'db> {
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        assert_eq!(var.kind(db), SymGenericKind::Type);
        SymTy::new(db, SymTyKind::Var(var))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymTyKind<'db> {
    /// `$Perm $Ty`, e.g., `shared String
    Perm(SymPerm<'db>, SymTy<'db>),

    /// `path[arg1, arg2]`, e.g., `Vec[String]`
    ///
    /// Important: the generic arguments must be well-kinded and of the correct number.
    Named(SymTyName<'db>, Vec<SymGenericTerm<'db>>),

    /// An inference variable (e.g., `?X`).
    Infer(InferVarIndex),

    /// Reference to a generic variable, e.g., `T`.
    Var(SymVariable<'db>),

    /// A value that can never be created, denoted `!`.
    Never,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymTyName<'db> {
    Primitive(SymPrimitive<'db>),

    Aggregate(SymAggregate<'db>),

    /// For now, just make future a builtin type
    #[no_from_impl]
    Future,

    #[no_from_impl]
    Tuple {
        arity: usize,
    },
}

impl std::fmt::Display for SymTyName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            match self {
                SymTyName::Primitive(primitive) => write!(f, "`{}`", primitive),
                SymTyName::Aggregate(class) => write!(f, "`{}`", class.name(db)),
                SymTyName::Tuple { arity } => write!(f, "{arity}-tuple"),
                SymTyName::Future => write!(f, "Future"),
            }
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[salsa::interned]
pub struct SymPerm<'db> {
    #[return_ref]
    pub kind: SymPermKind<'db>,
}

impl<'db> SymPerm<'db> {
    /// Returns the permission `my`.
    pub fn my(db: &'db dyn crate::Db) -> Self {
        SymPerm::new(db, SymPermKind::My)
    }

    /// Returns the permission `our`.
    pub fn our(db: &'db dyn crate::Db) -> Self {
        SymPerm::new(db, SymPermKind::Our)
    }
}

impl<'db> FromInfer<'db> for SymPerm<'db> {
    fn infer(db: &'db dyn crate::Db, var: InferVarIndex) -> Self {
        SymPerm::new(db, SymPermKind::Infer(var))
    }
}

impl<'db> std::fmt::Display for SymPerm<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}") // FIXME
    }
}

impl<'db> LeafBoundTerm<'db> for SymPerm<'db> {}

impl<'db> HasKind<'db> for SymPerm<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Perm
    }
}

impl<'db> FromVar<'db> for SymPerm<'db> {
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        assert_eq!(var.kind(db), SymGenericKind::Perm);
        SymPerm::new(db, SymPermKind::Var(var))
    }
}

impl<'db> Err<'db> for SymPerm<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        SymPerm::new(db, SymPermKind::Error(reported))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPermKind<'db> {
    My,
    Our,
    Shared(Vec<SymPlace<'db>>),
    Leased(Vec<SymPlace<'db>>),
    Given(Vec<SymPlace<'db>>),

    /// An inference variable (e.g., `?X`).
    Infer(InferVarIndex),

    Var(SymVariable<'db>),
    Error(Reported),
}

#[salsa::tracked]
pub struct SymPlace<'db> {
    #[return_ref]
    pub kind: SymPlaceKind<'db>,
}

impl<'db> SymPlace<'db> {
    pub fn field(self, db: &'db dyn crate::Db, field: SymField<'db>) -> Self {
        SymPlace::new(db, SymPlaceKind::Field(self, field))
    }
}

impl<'db> FromInfer<'db> for SymPlace<'db> {
    fn infer(db: &'db dyn crate::Db, var: InferVarIndex) -> Self {
        SymPlace::new(db, SymPlaceKind::Infer(var))
    }
}

impl<'db> std::fmt::Display for SymPlace<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}") // FIXME
    }
}

impl<'db> LeafBoundTerm<'db> for SymPlace<'db> {}

impl<'db> Err<'db> for SymPlace<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        SymPlace::new(db, SymPlaceKind::Error(reported))
    }
}

impl<'db> FromVar<'db> for SymPlace<'db> {
    fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        assert_eq!(var.kind(db), SymGenericKind::Place);
        SymPlace::new(db, SymPlaceKind::Var(var))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPlaceKind<'db> {
    /// `x`
    Var(SymVariable<'db>),

    /// `?x`
    Infer(InferVarIndex),

    /// `x.f`
    Field(SymPlace<'db>, SymField<'db>),

    /// `x[_]`
    Index(SymPlace<'db>),

    /// An error occurred and has been reported to the user.
    Error(Reported),
}

/// Create the symbol for an explicictly declared generic parameter.
/// This is tracked so that we do it at most once.
#[salsa::tracked]
impl<'db> IntoSymbol<'db> for AstGenericDecl<'db> {
    type Symbolic = SymVariable<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        SymVariable::new(
            db,
            self.kind(db).into_symbol(db),
            self.name(db).map(|n| n.id),
            self.span(db),
        )
    }
}

/// Convert to `SymGenericKind`
impl<'db> IntoSymbol<'db> for AstGenericKind<'db> {
    type Symbolic = SymGenericKind;

    fn into_symbol(self, _db: &'db dyn crate::Db) -> Self::Symbolic {
        match self {
            AstGenericKind::Type(_) => SymGenericKind::Type,
            AstGenericKind::Perm(_) => SymGenericKind::Perm,
        }
    }
}

pub(crate) trait AnonymousPermSymbol<'db> {
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db>;
}

/// Create the generic symbol for an anonymous permission like `shared T` or `leased T`.
/// This is desugared into the equivalent of `(perm:shared) T`.
///
/// Tracked so that it occurs at most once per `shared|leased|given` declaration.
#[salsa::tracked]
impl<'db> AnonymousPermSymbol<'db> for AstPerm<'db> {
    #[salsa::tracked]
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        match self.kind(db) {
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                SymVariable::new(db, SymGenericKind::Perm, None, self.span(db)).into()
            }
            _ => panic!("`anonymous_perm_symbol` invoked on inappropriate perm: {self:?}"),
        }
    }
}

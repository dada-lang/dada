use crate::{
    Db,
    ir::{
        binder::LeafBoundTerm,
        classes::{SymAggregate, SymField},
        indices::{FromInfer, FromInferVar, InferVarIndex},
        primitive::{SymPrimitive, SymPrimitiveKind},
        variables::{FromVar, SymVariable},
    },
    prelude::Symbol,
    well_known,
};
use dada_ir_ast::{
    ast::{AstGenericDecl, AstGenericKind, AstPerm, AstPermKind, AstSelfArg, VariableDecl},
    diagnostic::{Err, Errors, Reported},
    span::Spanned,
};
use dada_util::{FromImpls, SalsaSerialize};
use salsa::Update;
use serde::Serialize;

use super::classes::SymAggregateStyle;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct Variance {
    /// If true, then `T[P] <: T[Q]` requires `P <: Q` (necessary, not sufficient)
    pub at_least_covariant: bool,

    /// If true, then `T[P] <: T[Q]` requires `Q <: P` (necessary, not sufficient)
    pub at_least_contravariant: bool,

    /// Indicates that this type or permission
    /// is not directly owned by the struct/class but
    /// rather is relative to something else.
    ///
    /// Non-relative generics inherit permissions
    ///
    /// # Examples
    ///
    /// * `class Foo[type T](T)` -- `T` is NOT relative, so `our Foo[String]` becomes `our Foo[our String]`
    /// * `class Foo[perm P, type T](P T)` -- `P` is not relative, but `T` IS,
    ///   so `our Foo[my, String]` becomes `our Foo[our, String]`
    ///
    /// FIXME: Need a better name.
    pub relative: bool,
}

impl Variance {
    pub fn covariant() -> Self {
        Self {
            at_least_covariant: true,
            at_least_contravariant: false,
            relative: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum SymGenericKind {
    Type,
    Perm,
    Place,
}

impl std::fmt::Display for SymGenericKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "type"),
            Self::Perm => write!(f, "perm"),
            Self::Place => write!(f, "place"),
        }
    }
}

/// Test if `self` can be said to have the given kind (i.e., is it a type? a permission?).
///
/// Note that when errors occur, this may return true for multiple kinds.
pub trait HasKind<'db> {
    fn has_kind(&self, db: &'db dyn crate::Db, kind: SymGenericKind) -> bool;
}

/// Assert that `self` has the appropriate kind to produce an `R` value.
/// Implemented by e.g. [`SymGenericTerm`][] to permit downcasting to [`SymTy`](`crate::ir::ty::SymTy`).
pub trait AssertKind<'db, R> {
    fn assert_kind(self, db: &'db dyn crate::Db) -> R;
}

/// Value of a generic parameter
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls, Serialize,
)]
pub enum SymGenericTerm<'db> {
    Type(SymTy<'db>),
    Perm(SymPerm<'db>),
    Place(SymPlace<'db>),
    Error(Reported),
}

impl<'db> Err<'db> for SymGenericTerm<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        SymGenericTerm::Error(reported)
    }
}

impl std::fmt::Display for SymGenericTerm<'_> {
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
            SymGenericKind::Place => panic!("no inference variables for a place"),
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

    /// Returns the inference variable index if `self` is an inference variable.
    pub fn as_infer(self, db: &'db dyn crate::Db) -> Option<InferVarIndex> {
        match self {
            SymGenericTerm::Type(ty) => match ty.kind(db) {
                SymTyKind::Infer(infer) => Some(*infer),
                SymTyKind::Var(..)
                | SymTyKind::Named(..)
                | SymTyKind::Never
                | SymTyKind::Error(_)
                | SymTyKind::Perm(..) => None,
            },
            SymGenericTerm::Perm(perm) => match perm.kind(db) {
                SymPermKind::Infer(infer) => Some(*infer),
                SymPermKind::My
                | SymPermKind::Our
                | SymPermKind::Referenced(_)
                | SymPermKind::Mutable(_)
                | SymPermKind::Var(_)
                | SymPermKind::Error(_)
                | SymPermKind::Or(..)
                | SymPermKind::Apply(..) => None,
            },
            SymGenericTerm::Place(place) => match place.kind(db) {
                SymPlaceKind::Var(_)
                | SymPlaceKind::Erased
                | SymPlaceKind::Field(..)
                | SymPlaceKind::Index(..)
                | SymPlaceKind::Error(..) => None,
            },
            SymGenericTerm::Error(_) => None,
        }
    }

    /// Returns a string describing `self`, similar to "type `X`"
    pub fn describe(&self) -> String {
        match self {
            SymGenericTerm::Type(sym_ty) => format!("type `{sym_ty}`"),
            SymGenericTerm::Perm(sym_perm) => format!("permission `{sym_perm}`"),
            SymGenericTerm::Place(sym_place) => format!("place `{sym_place}`"),
            SymGenericTerm::Error(_) => "(error)".to_string(),
        }
    }
}

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub struct SymTy<'db> {
    #[return_ref]
    pub kind: SymTyKind<'db>,
}

impl<'db> SymTy<'db> {
    /// Returns a [`SymTyKind::Named`][] type for the given primitive type.
    pub fn primitive(db: &'db dyn Db, primitive: SymPrimitiveKind) -> Self {
        SymTy::named(db, primitive.intern(db).into(), vec![])
    }

    /// Returns a [`SymTyKind::Named`][] type for `u8`.
    pub fn u8(db: &'db dyn Db) -> Self {
        SymTy::primitive(db, SymPrimitiveKind::Uint { bits: 8 })
    }

    /// Returns a [`SymTyKind::Named`][] type for `u32`.
    pub fn u32(db: &'db dyn Db) -> Self {
        SymTy::primitive(db, SymPrimitiveKind::Uint { bits: 32 })
    }

    /// Returns a [`SymTyKind::Named`][] type for `String`.
    pub fn string(db: &'db dyn Db) -> Self {
        let string_class = match well_known::string_class(db) {
            Ok(v) => v,
            Err(reported) => return SymTy::err(db, reported),
        };
        SymTy::named(db, string_class.into(), vec![])
    }

    /// Returns a new [`SymTyKind::Named`][].
    pub fn named(
        db: &'db dyn Db,
        name: SymTyName<'db>,
        generics: Vec<SymGenericTerm<'db>>,
    ) -> Self {
        SymTy::new(db, SymTyKind::Named(name, generics))
    }

    /// Returns a [`SymTyKind::Var`][] type for the given variable.
    pub fn var(db: &'db dyn Db, var: SymVariable<'db>) -> Self {
        SymTy::new(db, SymTyKind::Var(var))
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
    pub fn referenced(self, db: &'db dyn Db, place: SymPlace<'db>) -> Self {
        SymTy::new(
            db,
            SymTyKind::Perm(SymPerm::referenced(db, vec![place]), self),
        )
    }

    /// Returns a version of this type mutable from `place`.
    pub fn mutable(self, db: &'db dyn Db, place: SymPlace<'db>) -> Self {
        SymTy::new(db, SymTyKind::Perm(SymPerm::mutable(db, vec![place]), self))
    }

    /// Returns a version of this type mutable from `place`.
    pub fn shared(self, db: &'db dyn Db) -> Self {
        SymTy::new(db, SymTyKind::Perm(SymPerm::our(db), self))
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
            SymTyKind::Perm(sym_perm, sym_ty) => write!(f, "{sym_perm} {sym_ty}"),
            SymTyKind::Infer(infer_var_index) => write!(f, "?{}", infer_var_index.as_usize()),
            SymTyKind::Var(sym_variable) => write!(f, "{sym_variable}"),
            SymTyKind::Never => write!(f, "!"),
            SymTyKind::Error(_) => write!(f, "<error>"),
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
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

#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls, Serialize,
)]
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

impl<'db> SymTyName<'db> {
    /// Aggregate style (struct, etc)
    pub fn style(self, db: &'db dyn crate::Db) -> SymAggregateStyle {
        match self {
            SymTyName::Primitive(_) => SymAggregateStyle::Struct,
            SymTyName::Aggregate(sym_aggregate) => sym_aggregate.style(db),
            SymTyName::Future => SymAggregateStyle::Class,
            SymTyName::Tuple { arity: _ } => SymAggregateStyle::Struct,
        }
    }
}

impl std::fmt::Display for SymTyName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            match self {
                SymTyName::Primitive(primitive) => write!(f, "{primitive}"),
                SymTyName::Aggregate(class) => write!(f, "{}", class.name(db)),
                SymTyName::Tuple { arity } => write!(f, "{arity}-tuple"),
                SymTyName::Future => write!(f, "Future"),
            }
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
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

    /// Returns a permission `shared` with the given places.
    pub fn referenced(db: &'db dyn crate::Db, places: Vec<SymPlace<'db>>) -> Self {
        SymPerm::new(db, SymPermKind::Referenced(places))
    }

    /// Returns a permission `mutable` with the given places.
    pub fn mutable(db: &'db dyn crate::Db, places: Vec<SymPlace<'db>>) -> Self {
        SymPerm::new(db, SymPermKind::Mutable(places))
    }

    /// Returns a generic permission with the given generic variable `var`.
    pub fn var(db: &'db dyn crate::Db, var: SymVariable<'db>) -> Self {
        SymPerm::new(db, SymPermKind::Var(var))
    }

    /// Returns a permission `perm1 perm2` (e.g., `shared[x] mutable[y]`).
    pub fn apply(db: &'db dyn crate::Db, perm1: SymPerm<'db>, perm2: SymPerm<'db>) -> Self {
        SymPerm::new(db, SymPermKind::Apply(perm1, perm2))
    }

    /// Returns a permission `perm1 | perm2`
    pub fn or(db: &'db dyn crate::Db, perm1: SymPerm<'db>, perm2: SymPerm<'db>) -> Self {
        SymPerm::new(db, SymPermKind::Or(perm1, perm2))
    }

    /// Apply this permission to the given term (if `self` is not `my`).
    pub fn apply_to<T>(self, db: &'db dyn crate::Db, term: T) -> T
    where
        T: Applicable<'db>,
    {
        match self.kind(db) {
            SymPermKind::My => term,
            _ => term.apply_from(db, self),
        }
    }
}

pub trait Applicable<'db> {
    fn apply_from(self, db: &'db dyn crate::Db, perm: SymPerm<'db>) -> Self;
}

impl<'db> Applicable<'db> for SymGenericTerm<'db> {
    fn apply_from(self, db: &'db dyn crate::Db, perm: SymPerm<'db>) -> Self {
        match self {
            SymGenericTerm::Type(ty) => ty.apply_from(db, perm).into(),
            SymGenericTerm::Perm(perm1) => perm1.apply_from(db, perm).into(),
            SymGenericTerm::Place(_) => panic!("cannot apply a perm to a place"),
            SymGenericTerm::Error(_) => self,
        }
    }
}

impl<'db> Applicable<'db> for SymTy<'db> {
    fn apply_from(self, db: &'db dyn crate::Db, perm: SymPerm<'db>) -> Self {
        SymTy::perm(db, perm, self)
    }
}

impl<'db> Applicable<'db> for SymPerm<'db> {
    fn apply_from(self, db: &'db dyn crate::Db, perm: SymPerm<'db>) -> Self {
        match self.kind(db) {
            SymPermKind::My => perm,
            _ => SymPerm::apply(db, perm, self),
        }
    }
}

impl<'db> FromInfer<'db> for SymPerm<'db> {
    fn infer(db: &'db dyn crate::Db, var: InferVarIndex) -> Self {
        SymPerm::new(db, SymPermKind::Infer(var))
    }
}

impl std::fmt::Display for SymPerm<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            match self.kind(db) {
                SymPermKind::My => write!(f, "my"),
                SymPermKind::Our => write!(f, "our"),
                SymPermKind::Referenced(places) => {
                    write!(f, "ref[")?;
                    for (i, place) in places.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{place}")?;
                    }
                    write!(f, "]")
                }
                SymPermKind::Mutable(places) => {
                    write!(f, "mutable[")?;
                    for (i, place) in places.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{place}")?;
                    }
                    write!(f, "]")
                }
                SymPermKind::Apply(perm1, perm2) => write!(f, "{perm1} {perm2}"),
                SymPermKind::Infer(infer_var_index) => write!(f, "?{}", infer_var_index.as_usize()),
                SymPermKind::Var(sym_variable) => write!(f, "{sym_variable}"),
                SymPermKind::Or(l, r) => write!(f, "({l} | {r})"),
                SymPermKind::Error(_) => write!(f, "<error>"),
            }
        })
        .unwrap_or_else(|| write!(f, "{self:?}"))
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum SymPermKind<'db> {
    /// `my`
    My,

    /// `our`
    Our,

    /// `ref[x]`
    Referenced(Vec<SymPlace<'db>>),

    /// `mutable[x]`
    Mutable(Vec<SymPlace<'db>>),

    /// `perm1 perm2` (e.g., `shared[x] mutable[y]`)
    Apply(SymPerm<'db>, SymPerm<'db>),

    /// An inference variable (e.g., `?X`).
    Infer(InferVarIndex),

    /// A generic variable (e.g., `T`).
    Var(SymVariable<'db>),

    /// Either `P | Q`
    Or(SymPerm<'db>, SymPerm<'db>),

    /// An error occurred and has been reported to the user.
    Error(Reported),
}

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct SymPlace<'db> {
    #[return_ref]
    pub kind: SymPlaceKind<'db>,
}

impl<'db> SymPlace<'db> {
    pub fn field(self, db: &'db dyn crate::Db, field: SymField<'db>) -> Self {
        SymPlace::new(db, SymPlaceKind::Field(self, field))
    }

    pub fn erased(db: &'db dyn crate::Db) -> Self {
        SymPlace::new(db, SymPlaceKind::Erased)
    }

    /// True if `self` contains no inference variables.
    pub fn no_inference_vars(self, db: &'db dyn crate::Db) -> bool {
        match self.kind(db) {
            SymPlaceKind::Var(..) => true,
            SymPlaceKind::Field(sym_place, _) => sym_place.no_inference_vars(db),
            SymPlaceKind::Index(sym_place) => sym_place.no_inference_vars(db),
            SymPlaceKind::Error(..) => true,
            SymPlaceKind::Erased => true,
        }
    }

    /// True if `self` *covers* `other`. Neither place may contain inference variables.
    ///
    /// # Definition
    ///
    /// A place P *covers* another place Q if P includes all of Q. E.g., `a` covers `a.b`.
    pub fn is_prefix_of(self, db: &'db dyn crate::Db, other: SymPlace<'db>) -> bool {
        assert!(self.no_inference_vars(db));
        assert!(other.no_inference_vars(db));
        self == other
            || match (self.kind(db), other.kind(db)) {
                (_, SymPlaceKind::Field(p2, _)) => self.is_prefix_of(db, *p2),
                _ => false,
            }
    }
}

impl std::fmt::Display for SymPlace<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            match self.kind(db) {
                SymPlaceKind::Var(var) => write!(f, "{var}"),
                SymPlaceKind::Field(place, field) => write!(f, "{place}.{field}"),
                SymPlaceKind::Index(place) => write!(f, "{place}[_]"),
                SymPlaceKind::Error(_) => write!(f, "<error>"),
                SymPlaceKind::Erased => write!(f, "_"),
            }
        })
        .unwrap_or_else(|| write!(f, "{self:?}"))
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum SymPlaceKind<'db> {
    /// `x`
    Var(SymVariable<'db>),

    /// `x.f`
    Field(SymPlace<'db>, SymField<'db>),

    /// `x[_]`
    Index(SymPlace<'db>),

    /// Erased place, used during codegen.
    /// Should never see it during type checking.
    Erased,

    /// An error occurred and has been reported to the user.
    Error(Reported),
}

/// Create the symbol for an explicictly declared generic parameter.
/// This is tracked so that we do it at most once.
#[salsa::tracked]
impl<'db> Symbol<'db> for AstGenericDecl<'db> {
    type Output = SymVariable<'db>;

    #[salsa::tracked]
    fn symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        SymVariable::new(
            db,
            self.kind(db).symbol(db),
            self.name(db).map(|n| n.id),
            self.span(db),
        )
    }
}

/// Convert to `SymGenericKind`
impl<'db> Symbol<'db> for AstGenericKind<'db> {
    type Output = SymGenericKind;

    fn symbol(self, _db: &'db dyn crate::Db) -> Self::Output {
        match self {
            AstGenericKind::Type(_) => SymGenericKind::Type,
            AstGenericKind::Perm(_) => SymGenericKind::Perm,
        }
    }
}

pub(crate) trait AnonymousPermSymbol<'db> {
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db>;
}

/// Create the generic symbol for an anonymous permission like `shared T` or `mutable T`.
/// This is desugared into the equivalent of `(perm:shared) T`.
///
/// Tracked so that it occurs at most once per `shared|mutable|given` declaration.
#[salsa::tracked]
impl<'db> AnonymousPermSymbol<'db> for AstPerm<'db> {
    #[salsa::tracked]
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        match self.kind(db) {
            AstPermKind::Referenced(None)
            | AstPermKind::Mutable(None)
            | AstPermKind::Given(None) => {
                SymVariable::new(db, SymGenericKind::Perm, None, self.span(db))
            }
            AstPermKind::Our
            | AstPermKind::Variable(_)
            | AstPermKind::GenericDecl(_)
            | AstPermKind::Referenced(Some(_))
            | AstPermKind::Mutable(Some(_))
            | AstPermKind::Given(Some(_))
            | AstPermKind::My => {
                panic!("`anonymous_perm_symbol` invoked on inappropriate perm: {self:?}")
            }
        }
    }
}

/// Create a generic symbol for an anonymous permission like `self`.
/// This is not always needed; see the implementation of [`PopulateSignatureSymbols`][]
/// for [`AstSelfArg`][].
///
/// Tracked so that it occurs at most once per `self` declaration.
#[salsa::tracked]
impl<'db> AnonymousPermSymbol<'db> for AstSelfArg<'db> {
    #[salsa::tracked]
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        assert!(self.perm(db).is_none());
        SymVariable::new(db, SymGenericKind::Perm, None, self.span(db))
    }
}

/// Create a generic symbol for a variable declaration that has no explicit
/// permission, like `x: String`. This is not always needed; see the
/// implementation of [`PopulateSignatureSymbols`][] for [`VariableDecl`][].
///
/// Tracked so that it occurs at most once per `self` declaration.
#[salsa::tracked]
impl<'db> AnonymousPermSymbol<'db> for VariableDecl<'db> {
    #[salsa::tracked]
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        assert!(self.perm(db).is_none());
        SymVariable::new(db, SymGenericKind::Perm, None, self.span(db))
    }
}

#[salsa::interned(debug)]
pub struct Assumption<'db> {
    pub kind: AssumptionKind,
    pub var: SymVariable<'db>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum AssumptionKind {
    Lent,
    Referenced,
    Unique,
    Mutable,
    Owned,
    Shared,
    My,
    Our,
}

use crate::{
    binder::LeafBoundTerm, class::SymAggregate, indices::{FromInferVar, InferVarIndex}, prelude::{IntoSymInScope, IntoSymbol}, primitive::SymPrimitive, scope::{NameResolution, NameResolutionSym, Resolve, Scope}, symbol::{AssertKind, FromVar, HasKind, SymGenericKind, SymVariable}, Db
};
use dada_ir_ast::{
    ast::{
        AstGenericDecl, AstGenericKind, AstGenericTerm, AstPath, AstPathKind, AstPerm, AstPermKind, AstTy, AstTyKind, Identifier
    },
    diagnostic::{ordinal, Diagnostic, Err, Errors, Level, Reported},
    span::{Span, Spanned},
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

impl<'db> LeafBoundTerm<'db> for SymGenericTerm<'db> {}

impl<'db> HasKind<'db> for SymGenericTerm<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        match self {
            SymGenericTerm::Type(_) => kind == SymGenericKind::Type,
            SymGenericTerm::Perm(_) => kind == SymGenericKind::Perm,
            SymGenericTerm::Place(_) =>kind == SymGenericKind::Place,
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

    pub fn kind(self) -> Errors<SymGenericKind> {
        match self {
            SymGenericTerm::Type(_) => Ok(SymGenericKind::Type),
            SymGenericTerm::Perm(_) => Ok(SymGenericKind::Perm),
            SymGenericTerm::Place(_) => Ok(SymGenericKind::Place),
            SymGenericTerm::Error(r) => Err(r),
        }
    }
}

#[salsa::interned]
pub struct SymTy<'db> {
    #[return_ref]
    pub kind: SymTyKind<'db>,
}

impl<'db> SymTy<'db> {
    /// Convenience constructor for named types
    pub fn named(db: &'db dyn Db, name: SymTyName<'db>, generics: Vec<SymGenericTerm<'db>>) -> Self {
        SymTy::new(db, SymTyKind::Named(name, generics))
    }

    /// Returns the type for `()`
    pub fn unit(db: &'db dyn Db) -> Self {
        #[salsa::tracked]
        fn unit_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
            SymTy::named(
                db,
                SymTyName::Tuple { arity: 0 },
                vec![]
            )
        }

        unit_ty(db)
    }

    /// Returns the type for `!`
    pub fn never(db: &'db dyn Db) -> Self {
        #[salsa::tracked]
        fn never_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
            SymTy::new(
                db,
                SymTyKind::Never,
            )
        }

        never_ty(db)
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
        }).unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[salsa::interned]
pub struct SymPerm<'db> {
    #[return_ref]
    pub kind: SymPermKind<'db>,
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
    Field(SymPlace<'db>, Identifier<'db>),

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

/// Convert an ast type into a symbolic type given the scope.
impl<'db> IntoSymInScope<'db> for AstTy<'db> {
    type Symbolic = SymTy<'db>;

    fn into_sym_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        let err = |r| SymTy::new(db, SymTyKind::Error(r));

        match self.kind(db) {
            AstTyKind::Perm(ast_perm, ast_ty) => {
                let perm = ast_perm.into_sym_in_scope(db, scope);
                let ty = ast_ty.into_sym_in_scope(db, scope);
                SymTy::new(db, SymTyKind::Perm(perm, ty))
            }
            AstTyKind::Named(ast_path, span_vec) => {
                let generics = span_vec
                    .iter()
                    .flatten()
                    .map(|g| (g.span(db), g.into_sym_in_scope(db, scope)))
                    .collect::<Vec<_>>();
                match ast_path.resolve_in(db, scope) {
                    Ok(r) => r.to_sym_ty(db, ast_path, generics),
                    Err(r) => err(r),
                }
            }
            AstTyKind::GenericDecl(decl) => {
                let symbol = decl.into_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Type);
                symbol
                    .into_generic_term(db, scope)
                    .assert_type(db)
            }
        }
    }
}

impl<'db> IntoSymInScope<'db> for AstGenericTerm<'db> {
    type Symbolic = SymGenericTerm<'db>;

    fn into_sym_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        match self {
            AstGenericTerm::Ty(ast_ty) => ast_ty.into_sym_in_scope(db, scope).into(),
            AstGenericTerm::Perm(ast_perm) => ast_perm.into_sym_in_scope(db, scope).into(),
            AstGenericTerm::Id(id) => match id.resolve_in(db, scope) {
                Ok(r) => r.to_sym_generic_arg(db, id),
                Err(r) => r.into(),
            },
        }
    }
}

impl<'db> IntoSymInScope<'db> for AstPerm<'db> {
    type Symbolic = SymPerm<'db>;

    fn into_sym_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        match self.kind(db) {
            AstPermKind::Shared(Some(places)) => {
                let places = places
                    .iter()
                    .map(|&p| path_to_place(db, scope, p))
                    .collect();
                SymPerm::new(db, SymPermKind::Shared(places))
            }
            AstPermKind::Leased(Some(places)) => {
                let places = places
                    .iter()
                    .map(|&p| path_to_place(db, scope, p))
                    .collect();
                SymPerm::new(db, SymPermKind::Leased(places))
            }
            AstPermKind::Given(Some(places)) => {
                let places = places
                    .iter()
                    .map(|&p| path_to_place(db, scope, p))
                    .collect();
                SymPerm::new(db, SymPermKind::Given(places))
            }
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                let symbol = self.anonymous_perm_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Perm);
                symbol.into_generic_term(db, scope).assert_perm(db)
            }
            AstPermKind::My => SymPerm::new(db, SymPermKind::My),
            AstPermKind::Our => SymPerm::new(db, SymPermKind::Our),
            AstPermKind::Variable(id) => match id.resolve_in(db, scope) {
                Ok(r) => r.to_sym_perm(db, *id).into(),
                Err(r) => SymPerm::new(db, SymPermKind::Error(r)),
            },
            AstPermKind::GenericDecl(decl) => {
                let symbol = decl.into_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Perm);
                symbol.into_generic_term(db, scope).assert_perm(db)
            }
        }
    }
}

impl<'db> NameResolution<'db> {
    /// Convert this name resolution into some kind of generic argument.
    /// This is called when we have something like `Foo[C]`;
    /// in that case, once we know what `C` is, we can decide if it is a type
    /// or a permission.
    pub(crate) fn to_sym_generic_arg(
        self,
        db: &'db dyn crate::Db,
        source: impl Spanned<'db>,
    ) -> SymGenericTerm<'db> {
        if let NameResolutionSym::SymVariable(var) = self.sym {
            match var.kind(db) {
                SymGenericKind::Type => SymGenericTerm::Type(self.to_sym_ty(db, source, vec![])),
                SymGenericKind::Perm => SymGenericTerm::Perm(self.to_sym_perm(db, source)),
                SymGenericKind::Place => SymGenericTerm::Place(self.to_sym_place(db, source)),
            }
        } else {
            self.to_sym_ty(db, source, vec![]).into()
        }
    }

    /// Convert this name resolution into a type; `generics` is the list of generic arguments that were supplied
    /// (if any).
    fn to_sym_ty(
        self,
        db: &'db dyn crate::Db,
        source: impl Spanned<'db>,
        generics: Vec<(Span<'db>, SymGenericTerm<'db>)>,
    ) -> SymTy<'db> {
        match self.sym {
            NameResolutionSym::SymPrimitive(sym_primitive) => {
                if generics.len() != 0 {
                    return SymTy::err(
                        db,
                        Diagnostic::error(
                            db,
                            source.span(db),
                            format!(
                                "`{}` does not expect generic arguments",
                                sym_primitive.name(db)
                            ),
                        )
                        .label(
                            db,
                            Level::Error,
                            source.span(db),
                            format!(
                                "the primitive type `{}` does not expect generic arguments",
                                sym_primitive.name(db)
                            ),
                        )
                        .report(db),
                    );
                }

                SymTy::named(db, sym_primitive.into(), vec![])
            }

            NameResolutionSym::SymClass(sym_class) => {
                let expected = sym_class.len_generics(db);
                let found = generics.len();
                if found != expected {
                    let name = sym_class.name(db);
                    return SymTy::err(
                        db,
                        Diagnostic::error(
                            db,
                            source.span(db),
                            format!("expected {expected} generic arguments, found {found}"),
                        )
                        .label(
                            db,
                            Level::Error,
                            source.span(db),
                            format!(
                            "`{name}` expects {expected} generic arguments, but I found {found}"
                        ),
                        )
                        .label(
                            db,
                            Level::Info,
                            sym_class.generics_span(db),
                            format!("generic arguments for `{name}` are declared here"),
                        )
                        .report(db),
                    );
                }

                let generics = sym_class
                    .generic_kinds(db)
                    .zip(&generics)
                    .zip(0..)
                    .map(|((expected_kind, &(span, generic)), index)| {
                        if generic.has_kind(db, expected_kind) {
                            generic
                        } else {
                            let found_kind = generic.kind().unwrap();
                            let name = sym_class.name(db);
                            SymGenericTerm::Error(
                                Diagnostic::error(
                                    db,
                                    span,
                                    format!("expected a `{expected_kind}`, found a `{found_kind}`"),
                                )
                                .label(
                                    db,
                                    Level::Error,
                                    span,
                                    format!(
                                        "`{name}` expects a `{expected_kind}` for its {ith} generic argument, but I found a `{found_kind}`",
                                        ith = ordinal(index + 1),
                                    ),
                                )
                                .label(
                                    db,
                                    Level::Info,
                                    sym_class.generic_span(db, index),
                                    format!(
                                        "{ith} generic argument for `{name}` is declared here",
                                        ith = ordinal(index + 1),
                                    ),
                                )
                                .report(db)    
                            )
                        }
                    })
                    .collect();

                SymTy::named(db, sym_class.into(), generics)
            }
            NameResolutionSym::SymVariable(var) => {
                if generics.len() != 0 {
                    return SymTy::err(
                        db,
                        Diagnostic::error(db, source.span(db), "generic types do not expect generic arguments")
                            .label(
                                db,
                                Level::Error,
                                source.span(db),
                                "this is the name of a generic type, but I also found a list of generic arguments",
                            )
                            .report(db),
                    );
                }

                let generic_kind = var.kind(db);
                if generic_kind != SymGenericKind::Type {
                    return SymTy::err(
                        db,
                        Diagnostic::error(db, source.span(db), format!("expected `type`, found `{generic_kind}`"))
                            .label(
                                db,
                                Level::Error,
                                source.span(db),
                                format!("I expected a type here, but I found a `{generic_kind}`"),
                            )
                            .report(db),
                    );
                }

                SymTy::var(db, var)
            }
            NameResolutionSym::SymModule(sym_module) => SymTy::err(
                db,
                Diagnostic::error(db, source.span(db), "modules are not valid types")
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!(
                            "I expected a type here, but `{}` is a module",
                            sym_module.name(db)
                        ),
                    )
                    .report(db),
            ),
            NameResolutionSym::SymFunction(sym_function) => SymTy::err(
                db,
                Diagnostic::error(db, source.span(db), "modules are not valid types")
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!(
                            "I expected a type here, but `{}` is a function",
                            sym_function.name(db)
                        ),
                    )
                    .report(db),
            ),
        }
    }

    /// Convert this name resolution into a permission.
    fn to_sym_perm(self, db: &'db dyn Db, source: impl Spanned<'db>) -> SymPerm<'db> {
        if let NameResolutionSym::SymVariable(var) = self.sym {
            if let SymGenericKind::Perm = var.kind(db) {
                return SymPerm::var(db, var);
            }
        }

        SymPerm::new(
            db,
            SymPermKind::Error(
                Diagnostic::error(db, source.span(db), "not a valid permission")
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!("I expected a permission here, but I found something else"),
                    )
                    .report(db),
            ),
        )
    }

    /// Convert this name resolution into a permission.
    fn to_sym_place(self, db: &'db dyn Db, source: impl Spanned<'db>) -> SymPlace<'db> {
        if let NameResolutionSym::SymVariable(var) = self.sym {
            if let SymGenericKind::Place = var.kind(db) {
                return SymPlace::var(db, var);
            }
        }

        SymPlace::new(
            db,
            SymPlaceKind::Error(
                Diagnostic::error(db, source.span(db), "not a valid place")
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!("I expected a place here, but I found something else"),
                    )
                    .report(db),
            ),
        )
    }
}

impl<'db> SymPlace<'db> {
    /// Create a new place expression extended with a field `field`.
    pub fn field(self, db: &'db dyn crate::Db, field: Identifier<'db>) -> Self {
        SymPlace::new(db, SymPlaceKind::Field(self, field))
    }
}

fn path_to_place<'db>(db: &'db dyn crate::Db, scope: &Scope<'_, 'db>, path: AstPath<'db>) -> SymPlace<'db> {
        match path.kind(db) {
            AstPathKind::Identifier(id) => {
                match scope.resolve_name(db, id.id, id.span) {
                    Ok(name) => match name.sym {
                        NameResolutionSym::SymVariable(var) => {
                            let var_kind = var.kind(db);
                            if var_kind == SymGenericKind::Place {
                                SymPlace::var(db, var)
                            } else {
                                SymPlace::new(
                                    db,
                                    SymPlaceKind::Error(
                                        Diagnostic::error(db, id.span, format!("expected a place, found `{var_kind}`"))
                                            .label(
                                                db,
                                                Level::Error,
                                                id.span,
                                                format!("I expected a place here, but I found a `{var_kind}`"),
                                            )
                                            .report(db),
                                    ),
                                )
                            }
                        }

                        _ => SymPlace::new(
                            db,
                            SymPlaceKind::Error(
                                Diagnostic::error(
                                    db,
                                    id.span(db),
                                    format!(
                                        "expected place expression, found {}",
                                        name.categorize(db)
                                    ),
                                )
                                .label(
                                    db,
                                    Level::Error,
                                    id.span(db),
                                    format!(
                                        "I expected a place expression, but I found {}",
                                        name.describe(db)
                                    ),
                                )
                                .report(db),
                            ),
                        ),
                    },
                    Err(r) => SymPlace::err(db, r),
                }
            }

            AstPathKind::GenericArgs { path: _, args } => {
                SymPlace::new(
                    db,
                    SymPlaceKind::Error(
                        Diagnostic::error(
                            db,
                            args.span,
                            format!(
                                "did not expect `[]` in place expression",
                            ),
                        )
                        .label(
                            db,
                            Level::Error,
                            args.span,
                            format!(
                                "I did not expect to find `[]` here",
                            ),
                        )
                        .report(db),
                    ),
                )
            }

            AstPathKind::Member { path, id } => SymPlace::new(db, SymPlaceKind::Field(path_to_place(db, scope, *path), id.id)),
        }
    }


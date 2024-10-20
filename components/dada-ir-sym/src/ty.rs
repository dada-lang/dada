use crate::{
    class::SymClass,
    indices::{SymBinderIndex, SymBoundVarIndex, SymInferVarIndex},
    prelude::{IntoSymInScope, IntoSymbol},
    primitive::SymPrimitive,
    scope::{NameResolution, Resolve, Scope},
    symbol::{HasKind, SymGenericKind, SymVariable},
    Db,
};
use dada_ir_ast::{
    ast::{
        AstGenericTerm, AstGenericDecl, AstGenericKind, AstPath, AstPerm, AstPermKind, AstTy,
        AstTyKind, Identifier,
    },
    diagnostic::{ordinal, Diagnostic, Errors, Level, Reported},
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

impl<'db> SymGenericTerm<'db> {
    pub fn var(db: &'db dyn crate::Db, kind: SymGenericKind, var: Var<'db>) -> Self {
        match kind {
            SymGenericKind::Type => SymTy::new(db, SymTyKind::Var(var)).into(),
            SymGenericKind::Perm => SymPerm::new(db, SymPermKind::Var(var)).into(),
            SymGenericKind::Place => SymPlace::new(db, SymPlaceKind::Var(var)).into(),
        }
    }

    pub fn assert_type(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        match self {
            SymGenericTerm::Type(ty) => ty,
            SymGenericTerm::Error(reported) => SymTy::new(db, SymTyKind::Error(reported)),
            _ => unreachable!(),
        }
    }

    pub fn assert_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        match self {
            SymGenericTerm::Perm(perm) => perm,
            SymGenericTerm::Error(reported) => SymPerm::new(db, SymPermKind::Error(reported)),
            _ => unreachable!(),
        }
    }

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
    /// Returns the type for `()`
    pub fn unit(db: &'db dyn Db) -> Self {
        #[salsa::tracked]
        fn unit_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
            SymTy::new(
                db,
                SymTyKind::Named(SymTyName::Tuple { arity: 0 }, Default::default()),
            )
        }

        unit_ty(db)
    }

    pub fn error(db: &'db dyn Db, reported: Reported) -> Self {
        SymTy::new(db, SymTyKind::Error(reported))
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

impl<'db> HasKind<'db> for SymTy<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Type
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

    /// Reference to a generic or inference variable, e.g., `T` or `?X`
    Var(Var<'db>),

    /// Indicates the user wrote `?` and we should use gradual typing.
    Unknown,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymTyName<'db> {
    Primitive(SymPrimitive<'db>),

    Class(SymClass<'db>),

    #[no_from_impl]
    Tuple {
        arity: usize,
    },
}

#[salsa::interned]
pub struct SymPerm<'db> {
    #[return_ref]
    pub kind: SymPermKind<'db>,
}

impl<'db> HasKind<'db> for SymPerm<'db> {
    fn has_kind(&self, _db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Perm
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPermKind<'db> {
    My,
    Our,
    Shared(Vec<SymPlace<'db>>),
    Leased(Vec<SymPlace<'db>>),
    Given(Vec<SymPlace<'db>>),
    Var(Var<'db>),
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum Var<'db> {
    /// A "universal" variable is meant to represent "any value" that meets its constraints.
    /// These are variables users declare, like generics, but also local variables (any value of the given type).
    Universal(SymVariable<'db>),

    /// An inference variable is one whose value is not yet determined.
    /// We accumulate constraints and try to find a value for it.
    Infer(SymInferVarIndex),

    /// A bound variable refers to a binder and is expected to be substituted.
    Bound(SymBinderIndex, SymBoundVarIndex),
}

#[salsa::tracked]
pub struct SymPlace<'db> {
    pub kind: SymPlaceKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPlaceKind<'db> {
    /// `x`
    Var(Var<'db>),

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
            AstTyKind::Unknown => SymTy::new(db, SymTyKind::Unknown),
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
                    .map(|p| p.into_sym_in_scope(db, scope))
                    .collect();
                SymPerm::new(db, SymPermKind::Shared(places))
            }
            AstPermKind::Leased(Some(places)) => {
                let places = places
                    .iter()
                    .map(|p| p.into_sym_in_scope(db, scope))
                    .collect();
                SymPerm::new(db, SymPermKind::Leased(places))
            }
            AstPermKind::Given(Some(places)) => {
                let places = places
                    .iter()
                    .map(|p| p.into_sym_in_scope(db, scope))
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
        if let NameResolution::SymVariable(var) = self {
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
        let make_named = |name, generics| SymTy::new(db, SymTyKind::Named(name, generics));
        let make_err = |r| SymTy::new(db, SymTyKind::Error(r));
        let make_var = |var| SymTy::new(db, SymTyKind::Var(var));

        match self {
            NameResolution::SymPrimitive(sym_primitive) => {
                if generics.len() != 0 {
                    return make_err(
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

                make_named(sym_primitive.into(), vec![])
            }

            NameResolution::SymClass(sym_class) => {
                let expected = sym_class.len_generics(db);
                let found = generics.len();
                if found != expected {
                    let name = sym_class.name(db);
                    return make_err(
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

                make_named(sym_class.into(), generics)
            }
            NameResolution::SymVariable(var) => {
                if generics.len() != 0 {
                    return make_err(
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
                    return make_err(
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

                make_var(Var::Universal(var))
            }
            NameResolution::SymModule(sym_module) => make_err(
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
            NameResolution::SymFunction(sym_function) => make_err(
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
    fn to_sym_perm(&self, db: &'db dyn Db, source: impl Spanned<'db>) -> SymPerm<'db> {
        if let &NameResolution::SymVariable(var) = self {
            if let SymGenericKind::Perm = var.kind(db) {
                return SymPerm::new(db, SymPermKind::Var(Var::Universal(var)));
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
    fn to_sym_place(&self, db: &'db dyn Db, source: impl Spanned<'db>) -> SymPlace<'db> {
        if let &NameResolution::SymVariable(var) = self {
            if let SymGenericKind::Place = var.kind(db) {
                return SymPlace::new(db, SymPlaceKind::Var(Var::Universal(var)));
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

impl<'db> IntoSymInScope<'db> for AstPath<'db> {
    type Symbolic = SymPlace<'db>;

    fn into_sym_in_scope(
        self,
        db: &'db dyn crate::Db,
        scope: &crate::scope::Scope<'_, 'db>,
    ) -> Self::Symbolic {
        let (first_id, other_ids) = self.ids(db).split_first().unwrap();

        // First resolve as many of the ids as we can using "lexical" resolution.
        // This will take care of any modules.
        let lexical_result = first_id
            .resolve_in(db, scope)
            .and_then(|r| r.resolve_relative(db, other_ids));

        // The final result `resolution` is what we attained via lexical resolution.
        // The slice `fields` are the remaining ids that are relative to this item.
        let (resolution, fields) = match lexical_result {
            Ok(pair) => pair,
            Err(reported) => return SymPlace::new(db, SymPlaceKind::Error(reported)),
        };

        // We expect the final resolution to be a local variable of some kind.
        // Anything else is an error.
        let NameResolution::SymVariable(var) = resolution else {
            return SymPlace::new(
                db,
                SymPlaceKind::Error(
                    Diagnostic::error(
                        db,
                        self.span(db),
                        format!(
                            "expected place expression, found {}",
                            resolution.categorize(db)
                        ),
                    )
                    .label(
                        db,
                        Level::Error,
                        self.span(db),
                        format!(
                            "I expected a place expression, but I found {}",
                            resolution.describe(db)
                        ),
                    )
                    .report(db),
                ),
            );
        };

        
        let var_kind = var.kind(db);
        if var_kind != SymGenericKind::Place {
            return SymPlace::new(
                db,
                SymPlaceKind::Error(
                    Diagnostic::error(db, first_id.span, format!("expected a place, found `{var_kind}`"))
                        .label(
                            db,
                            Level::Error,
                            first_id.span,
                            format!("I expected a place here, but I found a `{var_kind}`"),
                        )
                        .report(db),
                ),
            );
        }

        // Create the place. Note that in this phase we just include field ids with no closer examination.
        // The type checker must validate that they are correct.
        let mut place = SymPlace::new(db, SymPlaceKind::Var(Var::Universal(var)));
        for &id in fields {
            place = place.field(db, id.id);
        }
        place
    }
}

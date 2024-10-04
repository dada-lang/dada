use crate::{
    class::SymClass,
    indices::{SymBinderIndex, SymBoundVarIndex, SymExistentialVarIndex, SymUniversalVarIndex},
    prelude::IntoSymbol,
    scope::{NameResolution, Resolve, Scope},
    symbol::{SymGeneric, SymGenericKind, SymLocalVariable},
    Db, IntoSymInScope,
};
use dada_ir_ast::{
    ast::{
        AstGenericArg, AstGenericDecl, AstGenericKind, AstPerm, AstPermKind, AstTy, AstTyKind,
        Identifier,
    },
    diagnostic::Reported,
    diagnostic::{Diagnostic, Level},
    span::Span,
    span::Spanned,
};
use dada_util::FromImpls;
use salsa::Update;

/// Value of a generic parameter
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymGenericArg<'db> {
    Type(SymTy<'db>),
    Perm(SymPerm<'db>),
    Error(Reported),
}

#[salsa::interned]
pub struct SymTy<'db> {
    pub kind: SymTyKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymTyKind<'db> {
    Perm(SymPerm<'db>, SymTy<'db>),

    Named(SymTyName<'db>, Vec<SymGenericArg<'db>>),

    Var(GenericIndex),

    /// Indicates the user wrote `?` and we should use gradual typing.
    Unknown,

    /// Indicates some kind of error occurred and has been reported to the user.
    Error(Reported),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Binder<'db, T> {
    pub symbols: Vec<SymGeneric<'db>>,
    pub bound_value: T,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls)]
pub enum SymTyName<'db> {
    Class(SymClass<'db>),

    #[no_from_impl]
    Tuple {
        arity: usize,
    },
}

#[salsa::interned]
pub struct SymPerm<'db> {
    pub kind: SymPermKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPermKind<'db> {
    My,
    Our,
    Shared(Vec<SymPlace<'db>>),
    Leased(Vec<SymPlace<'db>>),
    Given(Vec<SymPlace<'db>>),
    Var(GenericIndex),
    Error(Reported),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum GenericIndex {
    Universal(SymUniversalVarIndex),
    Bound(SymBinderIndex, SymBoundVarIndex),
}

#[salsa::tracked]
pub struct SymPlace<'db> {
    pub kind: SymPlaceKind<'db>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPlaceKind<'db> {
    /// `x`
    LocalVariable(SymLocalVariable<'db>),

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
    type Symbolic = SymGeneric<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymGeneric<'db> {
        SymGeneric::new(
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
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymGeneric<'db>;
}

/// Create the generic symbol for an anonymous permission like `shared T` or `leased T`.
/// This is desugared into the equivalent of `(perm:shared) T`.
///
/// Tracked so that it occurs at most once per `shared|leased|given` declaration.
#[salsa::tracked]
impl<'db> AnonymousPermSymbol<'db> for AstPerm<'db> {
    #[salsa::tracked]
    fn anonymous_perm_symbol(self, db: &'db dyn crate::Db) -> SymGeneric<'db> {
        match self.kind(db) {
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                SymGeneric::new(db, SymGenericKind::Perm, None, self.span(db)).into()
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
                    .map(|g| g.into_sym_in_scope(db, scope))
                    .collect::<Vec<_>>();
                match ast_path.resolve_in(db, scope) {
                    Ok(r) => r.to_sym_ty(db, scope, ast_path, generics),
                    Err(r) => err(r),
                }
            }
            AstTyKind::GenericDecl(decl) => {
                let symbol = decl.into_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Type);
                scope
                    .resolve_generic_sym(db, symbol)
                    .to_sym_ty(db, scope, decl, vec![])
            }
            AstTyKind::Unknown => SymTy::new(db, SymTyKind::Unknown),
        }
    }
}

impl<'db> IntoSymInScope<'db> for AstGenericArg<'db> {
    type Symbolic = SymGenericArg<'db>;

    fn into_sym_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        match self {
            AstGenericArg::Ty(ast_ty) => ast_ty.into_sym_in_scope(db, scope).into(),
            AstGenericArg::Perm(ast_perm) => ast_perm.into_sym_in_scope(db, scope).into(),
            AstGenericArg::Id(id) => match id.resolve_in(db, scope) {
                Ok(r) => r.to_sym_generic_arg(db, scope, id),
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
                    .map(|p| p.into_place(db))
                    .collect();
                SymPerm::new(db, SymPermKind::Shared(places))
            }
            AstPermKind::Leased(Some(places)) => {
                let places = places
                    .iter()
                    .map(|p| p.into_sym_in_scope(db, scope))
                    .map(|p| p.into_place(db))
                    .collect();
                SymPerm::new(db, SymPermKind::Leased(places))
            }
            AstPermKind::Given(Some(places)) => {
                let places = places
                    .iter()
                    .map(|p| p.into_sym_in_scope(db, scope))
                    .map(|p| p.into_place(db))
                    .collect();
                SymPerm::new(db, SymPermKind::Given(places))
            }
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                let symbol = self.anonymous_perm_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Perm);
                scope
                    .resolve_generic_sym(db, symbol)
                    .to_sym_perm(db, scope, self)
            }
            AstPermKind::My => SymPerm::new(db, SymPermKind::My),
            AstPermKind::Our => SymPerm::new(db, SymPermKind::Our),
            AstPermKind::Variable(id) => match id.resolve_in(db, scope) {
                Ok(r) => r.to_sym_perm(db, scope, *id).into(),
                Err(r) => SymPerm::new(db, SymPermKind::Error(r)),
            },
            AstPermKind::GenericDecl(decl) => {
                let symbol = decl.into_symbol(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Perm);
                scope
                    .resolve_generic_sym(db, symbol)
                    .to_sym_perm(db, scope, self)
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
        scope: &Scope<'_, 'db>,
        source: impl Spanned<'db>,
    ) -> SymGenericArg<'db> {
        if let NameResolution::SymGeneric(generic, _) = self {
            match generic.kind(db) {
                SymGenericKind::Type => {
                    SymGenericArg::Type(self.to_sym_ty(db, scope, source, vec![]))
                }
                SymGenericKind::Perm => SymGenericArg::Perm(self.to_sym_perm(db, scope, source)),
            }
        } else {
            self.to_sym_ty(db, scope, source, vec![]).into()
        }
    }

    /// Convert this name resolution into a type; `generics` is the list of generic arguments that were supplied
    /// (if any).
    fn to_sym_ty(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        source: impl Spanned<'db>,
        generics: Vec<SymGenericArg<'db>>,
    ) -> SymTy<'db> {
        self.to_sym_ty_skel(
            db,
            scope,
            source,
            generics,
            |name, generics| SymTy::new(db, SymTyKind::Named(name, generics)),
            |generic_index| SymTy::new(db, SymTyKind::Var(generic_index)),
            |r| SymTy::new(db, SymTyKind::Error(r)),
        )
    }

    /// Helper function for creating a [`SymTy`][] or a [`SymTyc`][] from a [`NameResolution`][].
    /// The `make_*` functions create the appropriate result.
    pub(crate) fn to_sym_ty_skel<G, R>(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        source: impl Spanned<'db>,
        generics: Vec<G>,
        make_named: impl Fn(SymTyName<'db>, Vec<G>) -> R,
        make_var: impl Fn(GenericIndex) -> R,
        make_err: impl Fn(Reported) -> R,
    ) -> R {
        match self {
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

                make_named(sym_class.into(), generics)
            }
            NameResolution::SymGeneric(generic, generic_index) => {
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

                make_var(generic_index)
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
            NameResolution::SymLocalVariable(sym_local_variable) => make_err(
                Diagnostic::error(db, source.span(db), "modules are not valid types")
                    .label(
                        db,
                        Level::Error,
                        source.span(db),
                        format!(
                            "I expected a type here, but `{}` is a variable",
                            sym_local_variable.name(db)
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
    fn to_sym_perm(
        &self,
        db: &'db dyn Db,
        scope: &Scope<'_, 'db>,
        source: impl Spanned<'db>,
    ) -> SymPerm<'db> {
        if let NameResolution::SymGeneric(generic, generic_index) = self {
            if let SymGenericKind::Perm = generic.kind(db) {
                return SymPerm::new(db, SymPermKind::Var(*generic_index));
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
}

#[salsa::tracked]
impl<'db> SymTy<'db> {
    /// Returns the type for `()`
    pub fn unit(db: &'db dyn Db) -> Self {
        unit_ty(db)
    }
}

#[salsa::tracked]
fn unit_ty<'db>(db: &'db dyn Db) -> SymTy<'db> {
    SymTy::new(
        db,
        SymTyKind::Named(SymTyName::Tuple { arity: 0 }, Default::default()),
    )
}

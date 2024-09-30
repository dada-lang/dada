use dada_ir_ast::{
    ast::{AstGenericArg, AstGenericDecl, AstGenericKind, AstPerm, AstPermKind, AstTy, AstTyKind},
    diagnostic::{Diagnostic, Level},
    span::Spanned,
};

use crate::{
    prelude::Symbolize,
    scope::{GenericIndex, NameResolution, Resolve, Scope},
    symbol::{SymGeneric, SymGenericKind},
    ty::{SymGenericArg, SymPerm, SymTy, SymTyKind, SymTyName},
    Db, SymbolizeInScope,
};

#[salsa::tracked]
impl<'db> Symbolize<'db> for AstGenericDecl<'db> {
    type Symbolic = SymGeneric<'db>;

    #[salsa::tracked]
    fn symbolize(self, db: &'db dyn crate::Db) -> SymGeneric<'db> {
        SymGeneric::new(
            db,
            self.kind(db).symbolize(db),
            self.name(db).map(|n| n.id),
            self.span(db),
        )
    }
}

impl<'db> Symbolize<'db> for AstGenericKind<'db> {
    type Symbolic = SymGenericKind;

    fn symbolize(self, _db: &'db dyn crate::Db) -> Self::Symbolic {
        match self {
            AstGenericKind::Type(_) => SymGenericKind::Type,
            AstGenericKind::Perm(_) => SymGenericKind::Perm,
        }
    }
}

impl<'db> SymbolizeInScope<'db> for AstTy<'db> {
    type Symbolic = SymTy<'db>;

    fn symbolize_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        let err = |r| SymTy::new(db, SymTyKind::Error(r));

        match self.kind(db) {
            AstTyKind::Perm(ast_perm, ast_ty) => todo!(),
            AstTyKind::Named(ast_path, span_vec) => {
                let generics = span_vec
                    .iter()
                    .flatten()
                    .map(|g| g.symbolize_in_scope(db, scope))
                    .collect::<Vec<_>>();
                match ast_path.resolve_in(db, scope) {
                    Ok(r) => r.to_sym_ty(db, scope, ast_path, generics),
                    Err(r) => err(r),
                }
            }
            AstTyKind::GenericDecl(decl) => {
                let symbol = decl.symbolize(db);
                assert_eq!(symbol.kind(db), SymGenericKind::Type);
                scope
                    .resolve_generic_sym(db, symbol)
                    .to_sym_ty(db, scope, decl, vec![])
            }
            AstTyKind::Unknown => SymTy::new(db, SymTyKind::Unknown),
        }
    }
}

impl<'db> SymbolizeInScope<'db> for AstGenericArg<'db> {
    type Symbolic = SymGenericArg<'db>;

    fn symbolize_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        // WIP: the Scope should be carrying free indices

        match self {
            AstGenericArg::Ty(ast_ty) => ast_ty.symbolize_in_scope(db, scope).into(),
            AstGenericArg::Perm(ast_perm) => ast_perm.symbolize_in_scope(db, scope).into(),
            AstGenericArg::Id(spanned_identifier) => match spanned_identifier.resolve_in(db, scope)
            {
                Ok(r) => r.to_sym_ty(db, scope, spanned_identifier, vec![]).into(),
                Err(r) => r.into(),
            },
        }
    }
}

impl<'db> SymbolizeInScope<'db> for AstPerm<'db> {
    type Symbolic = SymPerm<'db>;

    fn symbolize_in_scope(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> Self::Symbolic {
        match self.kind(db) {
            AstPermKind::Shared(span_vec) => todo!(),
            AstPermKind::Leased(span_vec) => todo!(),
            AstPermKind::Given(span_vec) => todo!(),
            AstPermKind::My => todo!(),
            AstPermKind::Our => todo!(),
            AstPermKind::Variable(identifier) => todo!(),
            AstPermKind::GenericDecl(decl) => todo!(),
        }
    }
}

impl<'db> NameResolution<'db> {
    fn to_sym_ty(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        source: impl Spanned<'db>,
        generics: Vec<SymGenericArg<'db>>,
    ) -> SymTy<'db> {
        let err = |r| SymTy::new(db, SymTyKind::Error(r));
        match self {
            NameResolution::SymClass(sym_class) => {
                let expected = sym_class.len_generics(db);
                let found = generics.len();
                if found != expected {
                    let name = sym_class.name(db);
                    return err(Diagnostic::error(
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
                    .report(db));
                }
                SymTy::new(db, SymTyKind::Named(sym_class.into(), generics))
            }
            NameResolution::SymGeneric(generic, generic_index) => {
                if generics.len() != 0 {
                    return err(
                        Diagnostic::error(db, source.span(db), "generic types do not expect generic arguments")
                            .label(
                                db,
                                Level::Error,
                                source.span(db),
                                "This is the name of a generic type, but I also found a list of generic arguments",
                            )
                            .report(db),
                    );
                }

                match generic_index {
                    GenericIndex::Universal(sym_universal_var_index) => {
                        SymTy::new(db, SymTyKind::FreeUniversal(sym_universal_var_index))
                    }
                    GenericIndex::Bound(sym_debruijn_index, sym_bound_var_index) => todo!(),
                }
            }
            NameResolution::SymModule(sym_module) => {
                err(
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
                )
            }
            NameResolution::SymLocalVariable(sym_local_variable) => {
                err(
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
                )
            }
            NameResolution::SymFunction(sym_function) => {
                err(
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
                )
            }
        }
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

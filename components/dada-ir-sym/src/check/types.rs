use dada_ir_ast::{
    ast::{AstGenericTerm, AstPerm, AstPermKind, AstTy, AstTyKind}, diagnostic::{ordinal, Diagnostic, Err, Level}, span::{Span, Spanned}
};
use dada_util::indirect;

use crate::{check::{env::EnvLike, scope::{NameResolution, NameResolutionSym, Resolve}}, ir::{types::{AnonymousPermSymbol, HasKind, SymGenericKind, SymGenericTerm, SymPerm, SymPlace, SymTy}, variables::FromVar}, prelude::Symbol};

use super::CheckInEnvLike;

impl<'db> CheckInEnvLike<'db> for AstTy<'db> {
    type Output = SymTy<'db>;

    async fn check_in_env_like(self, env: &mut impl EnvLike<'db>) -> Self::Output {
        let db = env.db();
        indirect(async || {
            match self.kind(db) {
                AstTyKind::Perm(ast_perm, ast_ty) => {
                    let sym_perm = ast_perm.check_in_env_like(env).await;
                    let sym_ty = ast_ty.check_in_env_like(env).await;
                    SymTy::perm(db, sym_perm, sym_ty)
                }

                AstTyKind::Named(ast_path, ref opt_ast_generics) => {
                    let mut generics = vec![];
                    if let Some(ast_generics) = opt_ast_generics {
                        for g in ast_generics {
                            let span = g.span(db);
                            let checked = g.check_in_env_like(env).await;
                            generics.push((span, checked));
                        }
                    }
                    match ast_path.resolve_in(env).await {
                        Ok(r) => name_resolution_to_sym_ty(db, r, ast_path, generics),
                        Err(r) => SymTy::err(db, r),
                    }
                }

                AstTyKind::GenericDecl(decl) => {
                    let symbol = decl.symbol(db);
                    SymTy::var(db, symbol)
                }
            }
        }).await
    }
}

fn name_resolution_to_sym_ty<'db>(
    db: &'db dyn crate::Db,
    name_resolution: NameResolution<'db>,
    source: impl Spanned<'db>,
    generics: Vec<(Span<'db>, SymGenericTerm<'db>)>,
) -> SymTy<'db> {
    match name_resolution.sym {
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

impl<'db> CheckInEnvLike<'db> for AstGenericTerm<'db> {
    type Output = SymGenericTerm<'db>;

    async fn check_in_env_like(self, env: &mut impl EnvLike<'db>) -> Self::Output {
        match self {
            AstGenericTerm::Ty(ast_ty) => ast_ty.check_in_env_like(env).await.into(),
            AstGenericTerm::Perm(ast_perm) => ast_perm.check_in_env_like(env).await.into(),
            AstGenericTerm::Id(id) => match id.resolve_in(env).await {
                Ok(r) => name_resolution_to_generic_term(env.db(), r, id),
                Err(r) => r.into(),
            },
        }
    }
}

fn name_resolution_to_generic_term<'db>(db: &'db dyn crate::Db, name_resolution: NameResolution<'db>, source: impl Spanned<'db>) -> SymGenericTerm<'db> {
    if let NameResolutionSym::SymVariable(var) = name_resolution.sym {
        match var.kind(db) {
            SymGenericKind::Type => SymGenericTerm::Type(SymTy::var(db, var)),
            SymGenericKind::Perm => SymGenericTerm::Perm(SymPerm::var(db, var)),
            SymGenericKind::Place => SymGenericTerm::Place(SymPlace::var(db, var)),
        }
    } else {
        name_resolution_to_sym_ty(db, name_resolution, source, vec![]).into()
    }
}

impl<'db> CheckInEnvLike<'db> for AstPerm<'db> {
    type Output = SymPerm<'db>;

    async fn check_in_env_like(self, env: &mut impl EnvLike<'db>) -> Self::Output {
        let db = env.db();
        match *self.kind(db) {
            #[expect(unused_variables)]
            AstPermKind::Shared(Some(ref paths)) => {
                todo!()
            }
            #[expect(unused_variables)]
            AstPermKind::Leased(Some(ref span_vec)) => {
                todo!()
            }
            AstPermKind::Given(Some(ref _span_vec)) => todo!(),
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                let sym_var = self.anonymous_perm_symbol(db);
                SymPerm::var(db, sym_var)
            }
            AstPermKind::My => SymPerm::my(db),
            AstPermKind::Our => SymPerm::our(db),
            AstPermKind::Variable(id) => {
                match id.resolve_in(env).await {
                    Ok(r) => name_resolution_to_sym_perm(db, r, id),
                    Err(r) => SymPerm::err(db, r),
                }
            }
            AstPermKind::GenericDecl(ast_generic_decl) => {
                let symbol = ast_generic_decl.symbol(db);
                SymPerm::var(db, symbol)
            }
        }
    }
}

fn name_resolution_to_sym_perm<'db>(db: &'db dyn crate::Db, name_resolution: NameResolution<'db>, source: impl Spanned<'db>) -> SymPerm<'db> {
    match name_resolution.sym {
        NameResolutionSym::SymVariable(sym_variable) if sym_variable.has_kind(db, SymGenericKind::Perm) =>
            SymPerm::var(db, sym_variable),

        NameResolutionSym::SymModule(_) | 
        NameResolutionSym::SymClass(_) | 
        NameResolutionSym::SymFunction(_) | 
        NameResolutionSym::SymVariable(_) |
        NameResolutionSym::SymPrimitive(_) => {
            SymPerm::err(
                db, 
                Diagnostic::error(
                    db, 
                    source.span(db), 
                    format!("expected permission, found {}", name_resolution.sym.categorize(db))
                )
                .label(
                    db,
                    Level::Error,
                    source.span(db),
                    format!("I expected a permission, but I found {}", name_resolution.sym.describe(db)),
                )
                .report(db)
            )
        } 
    }
}
use dada_ir_ast::{
    ast::{AstGenericTerm, AstPerm, AstPermKind, AstTy, AstTyKind}, diagnostic::{ordinal, Diagnostic, Err, Level}, span::{Span, Spanned}
};

use crate::{env::EnvLike, scope::{NameResolution, NameResolutionSym, Resolve}, symbol::{FromVar, SymGenericKind}, ty::{SymGenericTerm, SymPerm, SymPlace, SymTy}, prelude::Symbol, CheckInEnv};

impl<'db> CheckInEnv<'db> for AstTy<'db> {
    type Output = SymTy<'db>;

    fn check_in_env(self, env: &mut dyn EnvLike<'db>) -> Self::Output {
        let db = env.db();
        match self.kind(db) {
            AstTyKind::Perm(ast_perm, ast_ty) => {
                let sym_perm = ast_perm.check_in_env(env);
                let sym_ty = ast_ty.check_in_env(env);
                SymTy::perm(db, sym_perm, sym_ty)
            }

            AstTyKind::Named(ast_path, generics) => {
                let generics = generics
                    .iter()
                    .flatten()
                    .map(|g| (g.span(db), g.check_in_env(env)))
                    .collect::<Vec<_>>();
                match ast_path.resolve_in(env) {
                    Ok(r) => name_resolution_to_sym_ty(db, r, ast_path, generics),
                    Err(r) => SymTy::err(db, r),
                }
            }

            AstTyKind::GenericDecl(decl) => {
                let symbol = decl.symbol(db);
                SymTy::var(db, symbol)
            }
        }
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

impl<'db> CheckInEnv<'db> for AstGenericTerm<'db> {
    type Output = SymGenericTerm<'db>;

    fn check_in_env(self, env: &mut dyn EnvLike<'db>) -> Self::Output {
        match self {
            AstGenericTerm::Ty(ast_ty) => ast_ty.check_in_env(env).into(),
            AstGenericTerm::Perm(ast_perm) => ast_perm.check_in_env(env).into(),
            AstGenericTerm::Id(id) => match id.resolve_in(env) {
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


impl<'db> CheckInEnv<'db> for AstPerm<'db> {
    type Output = SymPerm<'db>;

    fn check_in_env(self, env: &mut dyn EnvLike<'db>) -> Self::Output {
        let db = env.db();
        match *self.kind(db) {
            AstPermKind::Shared(ref _span_vec) => todo!(),
            AstPermKind::Leased(ref _span_vec) => todo!(),
            AstPermKind::Given(ref _span_vec) => todo!(),
            AstPermKind::My => SymPerm::my(db),
            AstPermKind::Our => SymPerm::our(db),
            AstPermKind::Variable(_spanned_identifier) => todo!(),
            AstPermKind::GenericDecl(_ast_generic_decl) => todo!(),
        }
    }
}

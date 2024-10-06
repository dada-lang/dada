use std::borrow::Cow;

use dada_ir_ast::{
    ast::{AstClassItem, AstFieldDecl, AstMember, Identifier, SpannedIdentifier},
    span::{Span, Spanned},
};
use dada_parser::prelude::*;
use dada_util::FromImpls;

use crate::{
    function::{SignatureSymbols, SymFunction},
    populate::PopulateSignatureSymbols,
    prelude::IntoSymbol,
    scope::{Scope, ScopeItem},
    ty::{Binder, SymTy, SymTyKind},
    IntoSymInScope,
};

#[salsa::tracked]
pub struct SymClass<'db> {
    scope: ScopeItem<'db>,
    source: AstClassItem<'db>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, FromImpls)]
pub enum SymClassMember<'db> {
    SymField(SymField<'db>),
    SymFunction(SymFunction<'db>),
}

#[salsa::tracked]
pub struct SymField<'db> {
    pub class: SymClass<'db>,
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
    pub source: AstFieldDecl<'db>,
}

impl<'db> Spanned<'db> for SymClass<'db> {
    fn span(&self, db: &'db dyn salsa::Database) -> dada_ir_ast::span::Span<'db> {
        self.source(db).name_span(db)
    }
}

#[salsa::tracked]
impl<'db> SymClass<'db> {
    pub fn name(&self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db)
    }

    /// Number of generic parameters
    pub fn len_generics(&self, db: &'db dyn crate::Db) -> usize {
        if let Some(generics) = self.source(db).generics(db) {
            generics.len()
        } else {
            0
        }
    }

    /// Span of the class name, typically used in diagnostics
    pub fn name_span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        self.source(db).name_span(db)
    }

    /// Span where generics are declared (possibly the name span, if there are no generics)
    pub fn generics_span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        if let Some(generics) = self.source(db).generics(db) {
            generics.span
        } else {
            self.name_span(db)
        }
    }

    /// Returns the base scope used to resolve the class members.
    /// Typically this is created by invoke [`Scope::new`][].
    pub(crate) fn class_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let mut signature_symbols = SignatureSymbols::new(self);
        self.source(db)
            .populate_signature_symbols(db, &mut signature_symbols);
        Scope::new(db, self.scope(db)).with_link(Cow::Owned(signature_symbols))
    }

    /// Returns the type of this class, referencing the generics that appear in `scope`.
    pub(crate) fn self_ty(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> SymTy<'db> {
        SymTy::new(
            db,
            SymTyKind::Named(
                self.into(),
                self.source(db)
                    .generics(db)
                    .iter()
                    .flatten()
                    .map(|g| g.into_symbol(db))
                    .map(|g| scope.resolve_generic_sym(db, g).to_sym_generic_arg(db, g))
                    .collect(),
            ),
        )
    }

    #[salsa::tracked]
    pub fn members(self, db: &'db dyn crate::Db) -> Vec<SymClassMember<'db>> {
        self.source(db)
            .members(db)
            .iter()
            .map(|m| match *m {
                AstMember::Field(ast_field_decl) => {
                    let SpannedIdentifier { span, id } = ast_field_decl.variable(db).name(db);
                    SymField::new(db, self, id, span, ast_field_decl).into()
                }
                AstMember::Function(ast_function) => {
                    SymFunction::new(db, self.into(), ast_function).into()
                }
            })
            .collect()
    }

    pub fn fields(self, db: &'db dyn crate::Db) -> impl Iterator<Item = SymField<'db>> {
        self.members(db).into_iter().filter_map(|m| match m {
            SymClassMember::SymField(f) => Some(f),
            _ => None,
        })
    }

    pub fn methods(self, db: &'db dyn crate::Db) -> impl Iterator<Item = SymFunction<'db>> {
        self.members(db).into_iter().filter_map(|m| match m {
            SymClassMember::SymFunction(f) => Some(f),
            _ => None,
        })
    }
}

#[salsa::tracked]
impl<'db> SymField<'db> {
    #[salsa::tracked]
    pub fn ty(self, db: &'db dyn crate::Db) -> Binder<'db, SymTy<'db>> {
        let scope = Scope::new(db, self.class(db));
        let ast_ty = self.source(db).variable(db).ty(db);
        let sym_ty = ast_ty.into_sym_in_scope(db, &scope);
        scope.into_bound(sym_ty)
    }
}

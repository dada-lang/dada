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
    prelude::{IntoSymInScope, IntoSymbol},
    scope::{Scope, ScopeItem},
    symbol::SymGenericKind,
    ty::{Binder, SymTy, SymTyKind},
};

#[salsa::tracked]
pub struct SymClass<'db> {
    /// The scope in which this class is declared.
    scope_item: ScopeItem<'db>,

    /// The AST for this class.
    source: AstClassItem<'db>,
}

#[salsa::tracked]
impl<'db> SymClass<'db> {
    /// Name of the class.
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

    /// Kinds of generic parameters
    pub fn generic_kinds(
        &self,
        db: &'db dyn crate::Db,
    ) -> impl Iterator<Item = SymGenericKind> + 'db {
        self.source(db)
            .generics(db)
            .iter()
            .flatten()
            .map(move |decl| decl.kind(db).into_symbol(db))
    }

    /// Span of the class name, typically used in diagnostics.
    /// Also returned by the [`Spanned`][] impl.
    pub fn name_span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
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

    /// Span where the `index`th generics are is (possibly the name span, if there are no generics)
    ///
    /// # Panics
    ///
    /// If `index` is not a valid generic index
    pub fn generic_span(&self, db: &'db dyn crate::Db, index: usize) -> Span<'db> {
        let Some(generic) = self.source(db).generics(db).iter().flatten().nth(index) else {
            panic!(
                "invalid generic index `{index}` for `{name}`",
                name = self.name(db)
            )
        };
        generic.span(db)
    }

    /// Returns the base scope used to resolve the class members.
    /// Typically this is created by invoke [`Scope::new`][].
    pub(crate) fn class_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let mut signature_symbols = SignatureSymbols::new(self);
        self.source(db)
            .populate_signature_symbols(db, &mut signature_symbols);
        self.scope_item(db)
            .into_scope(db)
            .with_link(Cow::Owned(signature_symbols))
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

    #[salsa::tracked(return_ref)]
    pub fn members(self, db: &'db dyn crate::Db) -> Vec<SymClassMember<'db>> {
        self.source(db)
            .members(db)
            .iter()
            .map(|m| match *m {
                AstMember::Field(ast_field_decl) => {
                    let SpannedIdentifier { span, id } = ast_field_decl.variable(db).name(db);
                    SymField::new(db, self.into(), id, span, ast_field_decl).into()
                }
                AstMember::Function(ast_function) => {
                    SymFunction::new(db, self.into(), ast_function).into()
                }
            })
            .collect()
    }

    pub fn fields(self, db: &'db dyn crate::Db) -> impl Iterator<Item = SymField<'db>> {
        self.members(db).iter().filter_map(|&m| match m {
            SymClassMember::SymField(f) => Some(f),
            _ => None,
        })
    }

    pub fn methods(self, db: &'db dyn crate::Db) -> impl Iterator<Item = SymFunction<'db>> {
        self.members(db).iter().filter_map(|&m| match m {
            SymClassMember::SymFunction(f) => Some(f),
            _ => None,
        })
    }
}

/// Symbol for a class member
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, FromImpls)]
pub enum SymClassMember<'db> {
    /// Class fields
    SymField(SymField<'db>),

    /// Class methods
    SymFunction(SymFunction<'db>),
}

/// Symbol for a field of a class, struct, or enum
#[salsa::tracked]
pub struct SymField<'db> {
    /// The item in which this field is declared.
    pub scope_item: ScopeItem<'db>,

    /// Field name
    pub name: Identifier<'db>,

    /// Span of field name. Also returned by [`Spanned`][] impl.
    pub name_span: Span<'db>,

    /// AST for field declaration
    pub source: AstFieldDecl<'db>,
}

#[salsa::tracked]
impl<'db> SymField<'db> {
    #[salsa::tracked]
    pub fn ty(self, db: &'db dyn crate::Db) -> Binder<SymTy<'db>> {
        let scope = self.scope_item(db).into_scope(db);
        let ast_ty = self.source(db).variable(db).ty(db);
        let sym_ty = ast_ty.into_sym_in_scope(db, &scope);
        scope.into_bound(db, sym_ty)
    }
}

impl<'db> Spanned<'db> for SymClass<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.name_span(db)
    }
}

impl<'db> Spanned<'db> for SymField<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> dada_ir_ast::span::Span<'db> {
        self.name_span(db)
    }
}

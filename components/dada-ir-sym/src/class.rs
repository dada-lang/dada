use std::borrow::Cow;

use dada_ir_ast::{
    ast::{AstAggregate, AstAggregateKind, AstFieldDecl, AstMember, Identifier, SpannedIdentifier},
    span::{Span, Spanned},
};
use dada_parser::prelude::*;
use dada_util::FromImpls;

use crate::{
    function::{SignatureSymbols, SymFunction, SymFunctionSource},
    populate::PopulateSignatureSymbols,
    scope::Scope,
    scope_tree::{ScopeItem, ScopeTreeNode},
    symbol::{SymGenericKind, SymVariable},
    ty::{SymTy, SymTyKind},
    IntoSymbol,
};

#[salsa::tracked]
pub struct SymAggregate<'db> {
    /// The scope in which this class is declared.
    super_scope: ScopeItem<'db>,

    /// The AST for this class.
    source: AstAggregate<'db>,
}

#[salsa::tracked]
impl<'db> SymAggregate<'db> {
    /// Name of the class.
    pub fn name(&self, db: &'db dyn salsa::Database) -> Identifier<'db> {
        self.source(db).name(db)
    }

    /// Aggregate style (struct, etc)
    pub fn style(self, db: &'db dyn crate::Db) -> SymAggregateStyle {
        match self.source(db).kind(db) {
            AstAggregateKind::Class => SymAggregateStyle::Class,
            AstAggregateKind::Struct => SymAggregateStyle::Struct,
        }
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

    #[salsa::tracked(return_ref)]
    pub(crate) fn symbols(self, db: &'db dyn crate::Db) -> SignatureSymbols<'db> {
        let mut signature_symbols = SignatureSymbols::new(self);
        self.source(db)
            .populate_signature_symbols(db, &mut signature_symbols);
        signature_symbols
    }

    /// Returns the base scope used to resolve the class members.
    /// Typically this is created by invoke [`Scope::new`][].
    pub(crate) fn class_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let symbols = self.symbols(db);
        assert!(symbols.input_variables.is_empty());
        self.super_scope(db)
            .into_scope(db)
            .with_link(self)
            .with_link(Cow::Borrowed(&symbols.generic_variables[..]))
    }

    /// Returns the type of this class, referencing the generics that appear in `scope`.
    pub fn self_ty(self, db: &'db dyn crate::Db, scope: &Scope<'_, 'db>) -> SymTy<'db> {
        SymTy::new(
            db,
            SymTyKind::Named(
                self.into(),
                self.source(db)
                    .generics(db)
                    .iter()
                    .flatten()
                    .map(|g| g.into_symbol(db))
                    .map(|g| g.into_generic_term(db, scope))
                    .collect(),
            ),
        )
    }

    #[salsa::tracked(return_ref)]
    pub fn members(self, db: &'db dyn crate::Db) -> Vec<SymClassMember<'db>> {
        // If the class is declared like `class Foo(x: u32, y: u32)` then we make a constructor `new`
        // and a field for each of those members
        let ctor_members = self.source(db).inputs(db).iter().flat_map(|inputs| {
            let ctor = SymFunction::new(
                db,
                self.into(),
                SymFunctionSource::Constructor(self, self.source(db)),
            )
            .into();

            let fields = inputs.iter().map(|field_decl| {
                SymField::new(
                    db,
                    self.into(),
                    field_decl.variable(db).name(db).id,
                    field_decl.variable(db).name(db).span,
                    *field_decl,
                )
                .into()
            });

            std::iter::once(ctor).chain(fields)
        });

        // Also include anything the user explicitly wrote
        let explicit_members = self.source(db).members(db).iter().map(|m| match *m {
            AstMember::Field(ast_field_decl) => {
                let SpannedIdentifier { span, id } = ast_field_decl.variable(db).name(db);
                SymField::new(db, self.into(), id, span, ast_field_decl).into()
            }
            AstMember::Function(ast_function) => {
                SymFunction::new(db, self.into(), ast_function.into()).into()
            }
        });

        ctor_members.chain(explicit_members).collect()
    }

    #[salsa::tracked]
    pub fn inherent_member(
        self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<SymClassMember<'db>> {
        self.members(db)
            .into_iter()
            .copied()
            .find(|m| m.has_name(db, id))
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

impl std::fmt::Display for SymAggregate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| write!(f, "{}", self.name(db)))
            .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

impl<'db> ScopeTreeNode<'db> for SymAggregate<'db> {
    fn direct_super_scope(self, db: &'db dyn crate::Db) -> Option<ScopeItem<'db>> {
        Some(self.super_scope(db))
    }

    fn direct_generic_parameters(self, db: &'db dyn crate::Db) -> &'db Vec<SymVariable<'db>> {
        &self.symbols(db).generic_variables
    }

    fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        self.class_scope(db)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SymAggregateStyle {
    Struct,
    Class,
}

/// Symbol for a class member
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, FromImpls)]
pub enum SymClassMember<'db> {
    /// Class fields
    SymField(SymField<'db>),

    /// Class methods
    SymFunction(SymFunction<'db>),
}

impl<'db> SymClassMember<'db> {
    /// True if this class member has the given name.
    pub fn has_name(self, db: &'db dyn crate::Db, id: Identifier<'db>) -> bool {
        match self {
            SymClassMember::SymField(f) => f.name(db) == id,
            SymClassMember::SymFunction(f) => f.name(db) == id,
        }
    }
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
    /// The symbol for the `self` variable that appears in this field's type.
    /// (Every field and class member has their own `self` symbol.)
    #[salsa::tracked]
    pub fn self_sym(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        SymVariable::new(
            db,
            SymGenericKind::Place,
            Some(Identifier::self_ident(db)),
            self.name_span(db),
        )
    }

    /// The scope for resolving the type of this field.
    pub fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let self_sym = self.self_sym(db);
        self.scope_item(db).into_scope(db).with_link(self_sym)
    }
}

impl<'db> Spanned<'db> for SymAggregate<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.name_span(db)
    }
}

impl<'db> Spanned<'db> for SymField<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> dada_ir_ast::span::Span<'db> {
        self.name_span(db)
    }
}

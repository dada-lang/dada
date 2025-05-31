use std::borrow::Cow;

use dada_ir_ast::{
    ast::{
        AstAggregate, AstFunction, AstFunctionEffects, AstFunctionInput, AstMainFunction,
        Identifier, SpannedIdentifier,
    },
    span::{SourceSpanned, Span, Spanned},
};
use dada_util::{FromImpls, SalsaSerialize};
use salsa::Update;
use serde::Serialize;

use crate::{
    check::{
        scope::Scope,
        scope_tree::{ScopeItem, ScopeTreeNode},
    },
    ir::{
        binder::{Binder, LeafBoundTerm},
        classes::SymAggregate,
        populate::{PopulateDefaultSymbols, PopulateSignatureSymbols},
        types::SymTy,
        variables::SymVariable,
    },
};

use super::{
    classes::SymAggregateStyle,
    generics::SymWhereClause,
    types::{HasKind, SymGenericKind},
};

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct SymFunction<'db> {
    pub super_scope_item: ScopeItem<'db>,

    #[tracked]
    pub source: SymFunctionSource<'db>,
}

#[salsa::tracked]
impl<'db> SymFunction<'db> {
    #[salsa::tracked]
    pub fn effects(self, db: &'db dyn crate::Db) -> SymFunctionEffects {
        let source = self.source(db).effects(db);
        SymFunctionEffects {
            async_effect: source.async_effect.is_some(),
        }
    }
}

impl<'db> ScopeTreeNode<'db> for SymFunction<'db> {
    fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        self.scope(db)
    }

    fn direct_super_scope(self, db: &'db dyn crate::Db) -> Option<ScopeItem<'db>> {
        Some(self.super_scope_item(db))
    }

    fn direct_generic_parameters(self, db: &'db dyn crate::Db) -> &'db Vec<SymVariable<'db>> {
        &self.symbols(db).generic_variables
    }

    fn push_direct_ast_where_clauses(
        self,
        db: &'db dyn crate::Db,
        out: &mut Vec<dada_ir_ast::ast::AstWhereClause<'db>>,
    ) {
        let wc = match self.source(db) {
            SymFunctionSource::Function(ast) => ast.where_clauses(db),
            SymFunctionSource::Constructor(_, ast) => ast.where_clauses(db),
            SymFunctionSource::MainFunction(_) => &None,
        };

        if let Some(wc) = wc {
            out.extend(wc.clauses(db));
        }
    }
}

impl<'db> Spanned<'db> for SymFunction<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).name(db).span
    }
}

impl<'db> SourceSpanned<'db> for SymFunction<'db> {
    fn source_span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).source_span(db)
    }
}

#[salsa::tracked]
impl<'db> SymFunction<'db> {
    /// Name of the function.
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db).id
    }

    /// Span for the function name.
    pub fn name_span(self, db: &'db dyn crate::Db) -> Span<'db> {
        self.source(db).name(db).span
    }

    fn scope_from_symbols<'sym>(
        self,
        db: &'db dyn crate::Db,
        symbols: &'sym SignatureSymbols<'db>,
    ) -> Scope<'sym, 'db> {
        self.super_scope_item(db)
            .into_scope(db)
            .with_link(Cow::Borrowed(&symbols.generic_variables[..]))
            .with_link(Cow::Borrowed(&symbols.input_variables[..]))
    }

    #[salsa::tracked(return_ref)]
    pub fn symbols(self, db: &'db dyn crate::Db) -> SignatureSymbols<'db> {
        let source = self.source(db);

        // Before we can populate the default symbols,
        // we need to create a temporary scope with *just* the explicit symbols.
        // This allows us to do name resolution on the names of types and things.
        let mut just_explicit_symbols = SignatureSymbols::new(self);
        source.populate_signature_symbols(db, &mut just_explicit_symbols);
        let scope = self.scope_from_symbols(db, &just_explicit_symbols);

        // Now add in any default symbols.
        let mut with_default_symbols = just_explicit_symbols.clone();
        source.populate_default_symbols(db, &scope, &mut with_default_symbols);

        with_default_symbols
    }

    /// Returns the scope for this function; this has the function generics
    /// and parameters in scope.
    pub fn scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let symbols = self.symbols(db);
        self.scope_from_symbols(db, symbols)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Update, FromImpls, Serialize)]
pub enum SymFunctionSource<'db> {
    Function(AstFunction<'db>),

    /// Generated `fn main()` from statements appearing at the top of a module
    MainFunction(AstMainFunction<'db>),

    /// Generated constructor from an aggregate like `struct Foo(x: u32)`
    #[no_from_impl] // I'd prefer to be explicit
    Constructor(SymAggregate<'db>, AstAggregate<'db>),
}

impl<'db> SymFunctionSource<'db> {
    fn effects(self, db: &'db dyn crate::Db) -> AstFunctionEffects<'db> {
        match self {
            Self::Function(ast_function) => ast_function.effects(db),
            Self::MainFunction(_) | Self::Constructor(..) => AstFunctionEffects::default(),
        }
    }

    fn name(self, db: &'db dyn dada_ir_ast::Db) -> SpannedIdentifier<'db> {
        match self {
            Self::Function(ast_function) => ast_function.name(db),
            Self::Constructor(class, _) => SpannedIdentifier {
                span: class.name_span(db),
                id: Identifier::new_ident(db),
            },
            Self::MainFunction(mfunc) => SpannedIdentifier {
                span: mfunc.statements(db).span,
                id: Identifier::main(db),
            },
        }
    }

    pub fn inputs(self, db: &'db dyn crate::Db) -> Cow<'db, [AstFunctionInput<'db>]> {
        match self {
            Self::Function(ast_function) => Cow::Borrowed(&ast_function.inputs(db).values),
            Self::Constructor(_, class) => Cow::Owned(
                class
                    .inputs(db)
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|i| i.variable(db).into())
                    .collect::<Vec<_>>(),
            ),
            Self::MainFunction(_) => Cow::Borrowed(&[]),
        }
    }
}

impl<'db> SourceSpanned<'db> for SymFunctionSource<'db> {
    fn source_span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        match self {
            SymFunctionSource::Function(ast_function) => ast_function.span(db),
            SymFunctionSource::Constructor(_, ast_aggregate) => ast_aggregate.span(db),
            SymFunctionSource::MainFunction(mfunc) => mfunc.span(db),
        }
    }
}

/// Set of effects that can be declared on the function.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SymFunctionEffects {
    pub async_effect: bool,
}

#[derive(SalsaSerialize)]
#[salsa::tracked(debug)]
pub struct SymFunctionSignature<'db> {
    #[return_ref]
    pub symbols: SignatureSymbols<'db>,

    /// Input/output types:
    ///
    /// * Outer binder is for generic symbols from the function and its surrounding scopes
    /// * Inner binder is the function local variables.
    #[return_ref]
    pub input_output: Binder<'db, Binder<'db, SymInputOutput<'db>>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub struct SymInputOutput<'db> {
    pub input_tys: Vec<SymTy<'db>>,
    pub output_ty: SymTy<'db>,
    pub where_clauses: Vec<SymWhereClause<'db>>,
}

impl<'db> LeafBoundTerm<'db> for SymInputOutput<'db> {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Update, Serialize)]
pub struct SignatureSymbols<'db> {
    /// Source of these symbols
    pub source: SignatureSource<'db>,

    /// Generic parmaeters on the class or function (concatenated)
    pub generic_variables: Vec<SymVariable<'db>>,

    /// Symbols for the function input variables (if any)
    pub input_variables: Vec<SymVariable<'db>>,
}

impl<'db> SignatureSymbols<'db> {
    pub fn has_generics_of_kind(&self, db: &'db dyn crate::Db, kinds: &[SymGenericKind]) -> bool {
        if self.generic_variables.len() != kinds.len() {
            return false;
        }
        self.generic_variables
            .iter()
            .zip(kinds)
            .all(|(&v, &k)| v.has_kind(db, k))
    }
}

impl<'db> SignatureSymbols<'db> {
    /// Create an empty set of signature symbols from a given source.
    /// The actual symbols themselves are populated via the trait
    /// [`PopulateSignatureSymbols`][].
    pub fn new(source: impl Into<SignatureSource<'db>>) -> Self {
        Self {
            source: source.into(),
            generic_variables: Vec::new(),
            input_variables: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Update, FromImpls, Serialize)]
pub enum SignatureSource<'db> {
    Class(SymAggregate<'db>),
    Function(SymFunction<'db>),
}

impl<'db> SignatureSource<'db> {
    pub fn aggr_style(self, db: &'db dyn crate::Db) -> Option<SymAggregateStyle> {
        match self {
            SignatureSource::Class(aggr) => Some(aggr.style(db)),
            SignatureSource::Function(_) => None,
        }
    }
}

use std::borrow::Cow;

use dada_ir_ast::{
    ast::{
        AstAggregate, AstFunction, AstFunctionEffects, AstFunctionInput, Identifier,
        SpannedIdentifier,
    },
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    check::scope::Scope,
    check::scope_tree::{ScopeItem, ScopeTreeNode},
    ir::binder::{Binder, LeafBoundTerm},
    ir::classes::SymAggregate,
    ir::populate::PopulateSignatureSymbols,
    ir::types::SymTy,
    ir::variables::SymVariable,
};

use super::types::{HasKind, SymGenericKind};

#[salsa::tracked]
pub struct SymFunction<'db> {
    pub super_scope_item: ScopeItem<'db>,
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
}

impl<'db> Spanned<'db> for SymFunction<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).name(db).span
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

    #[salsa::tracked(return_ref)]
    pub fn symbols(self, db: &'db dyn crate::Db) -> SignatureSymbols<'db> {
        let source = self.source(db);
        let mut symbols = SignatureSymbols::new(self);
        source.populate_signature_symbols(db, &mut symbols);
        symbols
    }

    /// Returns the scope for this function; this has the function generics
    /// and parameters in scope.
    pub fn scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let symbols = self.symbols(db);
        self.super_scope_item(db)
            .into_scope(db)
            .with_link(Cow::Borrowed(&symbols.generic_variables[..]))
            .with_link(Cow::Borrowed(&symbols.input_variables[..]))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Update, FromImpls)]
pub enum SymFunctionSource<'db> {
    Function(AstFunction<'db>),

    /// Generated constructor from an aggregate like `struct Foo(x: u32)`
    #[no_from_impl] // I'd prefer to be explicit
    Constructor(SymAggregate<'db>, AstAggregate<'db>),
}

impl<'db> SymFunctionSource<'db> {
    fn effects(self, db: &'db dyn crate::Db) -> AstFunctionEffects<'db> {
        match self {
            Self::Function(ast_function) => ast_function.effects(db),
            Self::Constructor(..) => AstFunctionEffects::default(),
        }
    }

    fn name(self, db: &'db dyn dada_ir_ast::Db) -> SpannedIdentifier<'db> {
        match self {
            Self::Function(ast_function) => ast_function.name(db),
            Self::Constructor(class, _) => SpannedIdentifier {
                span: class.name_span(db),
                id: Identifier::new_ident(db),
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
        }
    }

    fn populate_signature_symbols(
        self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        match self {
            Self::Function(ast_function) => ast_function.populate_signature_symbols(db, symbols),
            Self::Constructor(..) => {
                self.inputs(db)
                    .iter()
                    .for_each(|i| i.populate_signature_symbols(db, symbols));
            }
        }
    }
}

/// Set of effects that can be declared on the function.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SymFunctionEffects {
    pub async_effect: bool,
}

#[salsa::tracked]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymInputOutput<'db> {
    pub input_tys: Vec<SymTy<'db>>,

    pub output_ty: SymTy<'db>,
}

impl<'db> LeafBoundTerm<'db> for SymInputOutput<'db> {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Update)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Update, FromImpls)]
pub enum SignatureSource<'db> {
    Class(SymAggregate<'db>),
    Function(SymFunction<'db>),

    #[no_from_impl]
    Dummy,
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

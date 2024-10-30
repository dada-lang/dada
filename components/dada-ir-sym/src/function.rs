use std::borrow::Cow;

use dada_ir_ast::{
    ast::{
        AstClassItem, AstFunction, AstFunctionEffects, AstFunctionInput, Identifier,
        SpannedIdentifier,
    },
    diagnostic::Diagnostic,
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    binder::{Binder, LeafBoundTerm},
    class::SymClass,
    populate::PopulateSignatureSymbols,
    prelude::IntoSymInScope,
    scope::Scope,
    scope_tree::{ScopeItem, ScopeTreeNode},
    symbol::SymVariable,
    ty::{SymTy, SymTyKind, SymTyName},
};

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

    /// Function signature
    #[salsa::tracked(return_ref)]
    pub fn signature(self, db: &'db dyn crate::Db) -> SymFunctionSignature<'db> {
        let scope = self.scope(db);

        let mut input_output = SymInputOutput {
            input_tys: self
                .source(db)
                .inputs(db)
                .iter()
                .map(|i| input_ty(db, &scope, i))
                .collect(),

            output_ty: self
                .source(db)
                .output_ty_in_scope(db, &scope)
                .unwrap_or_else(|| SymTy::unit(db)),
        };

        if self.source(db).effects(db).async_effect.is_some() {
            input_output.output_ty =
                SymTy::named(db, SymTyName::Future, vec![input_output.output_ty.into()]);
        }

        let bound_input_output = scope.into_bound_value(db, input_output);

        SymFunctionSignature::new(db, self.symbols(db).clone(), bound_input_output)
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

    /// A class with declared input.
    #[no_from_impl] // I'd prefer to be explicit
    ClassConstructor(SymClass<'db>, AstClassItem<'db>),
}

impl<'db> SymFunctionSource<'db> {
    fn effects(self, db: &'db dyn crate::Db) -> AstFunctionEffects<'db> {
        match self {
            Self::Function(ast_function) => ast_function.effects(db),
            Self::ClassConstructor(..) => AstFunctionEffects::default(),
        }
    }

    fn name(self, db: &'db dyn dada_ir_ast::Db) -> SpannedIdentifier<'db> {
        match self {
            Self::Function(ast_function) => ast_function.name(db),
            Self::ClassConstructor(class, _) => SpannedIdentifier {
                span: class.name_span(db),
                id: Identifier::new_ident(db),
            },
        }
    }

    fn inputs(self, db: &'db dyn crate::Db) -> Cow<'db, [AstFunctionInput<'db>]> {
        match self {
            Self::Function(ast_function) => Cow::Borrowed(&ast_function.inputs(db).values),
            Self::ClassConstructor(_, class) => Cow::Owned(
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
            Self::ClassConstructor(..) => {
                self.inputs(db)
                    .iter()
                    .for_each(|i| i.populate_signature_symbols(db, symbols));
            }
        }
    }

    fn output_ty_in_scope(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Option<SymTy<'db>> {
        match self {
            Self::Function(ast_function) => {
                let ast_ty = ast_function.output_ty(db)?;
                Some(ast_ty.into_sym_in_scope(db, &scope))
            }
            Self::ClassConstructor(sym_class, _) => Some(sym_class.self_ty(db, scope)),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Update, FromImpls)]
pub enum SignatureSource<'db> {
    Class(SymClass<'db>),
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

fn input_ty<'db>(
    db: &'db dyn crate::Db,
    scope: &Scope<'_, 'db>,
    input: &AstFunctionInput<'db>,
) -> SymTy<'db> {
    match input {
        AstFunctionInput::SelfArg(ast_self_arg) => match scope.class() {
            Some(class) => {
                let class_ty = class.self_ty(db, scope);
                if let Some(ast_perm) = ast_self_arg.perm(db) {
                    let perm = ast_perm.into_sym_in_scope(db, scope);
                    SymTy::new(db, SymTyKind::Perm(perm, class_ty))
                } else {
                    class_ty
                }
            }
            None => SymTy::new(
                db,
                SymTyKind::Error(
                    Diagnostic::error(
                        db,
                        ast_self_arg.self_span(db),
                        "cannot use `self` outside of a class",
                    )
                    .label(
                        db,
                        dada_ir_ast::diagnostic::Level::Error,
                        ast_self_arg.self_span(db),
                        "I did not expect a `self` parameter outside of a class definition",
                    )
                    .report(db),
                ),
            ),
        },
        AstFunctionInput::Variable(variable_decl) => {
            variable_decl.ty(db).into_sym_in_scope(db, scope)
        }
    }
}

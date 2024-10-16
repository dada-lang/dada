use std::borrow::Cow;

use dada_ir_ast::{
    ast::{AstBlock, AstFunction, AstFunctionInput, Identifier},
    diagnostic::Diagnostic,
    span::{Span, Spanned},
};
use dada_parser::prelude::FunctionBlock;
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymClass,
    populate::PopulateSignatureSymbols,
    prelude::IntoSymInScope,
    scope::{Scope, ScopeItem},
    symbol::SymVariable,
    ty::{Binder, SymTy, SymTyKind},
};

#[salsa::tracked]
pub struct SymFunction<'db> {
    scope_item: ScopeItem<'db>,
    source: AstFunction<'db>,
}

impl<'db> Spanned<'db> for SymFunction<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).name(db).span
    }
}

#[salsa::tracked]
pub struct SymFunctionSignature<'db> {
    #[return_ref]
    pub symbols: SignatureSymbols<'db>,

    /// Input/output types:
    ///
    /// * Outermost binder is the class (if a standalone function, this is empty).
    /// * Middle binder is the function generic types.
    /// * Inner binder is the function local variables.
    pub input_output: Binder<Binder<Binder<SymInputOutput<'db>>>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymInputOutput<'db> {
    pub input_tys: Vec<SymTy<'db>>,

    pub output_ty: SymTy<'db>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Update)]
pub struct SignatureSymbols<'db> {
    pub source: SignatureSource<'db>,
    pub generics: Vec<SymVariable<'db>>,
    pub inputs: Vec<SymVariable<'db>>,
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
            generics: Vec::new(),
            inputs: Vec::new(),
        }
    }
}

impl<'db> SymFunctionSignature<'db> {
    pub fn inputs(self, db: &'db dyn crate::Db) -> &'db [SymVariable<'db>] {
        &self.symbols(db).inputs
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

    /// Access the AST for this function.
    pub fn ast_body(self, db: &'db dyn crate::Db) -> Option<AstBlock<'db>> {
        self.source(db).body_block(db)
    }

    #[salsa::tracked(return_ref)]
    pub fn symbols(self, db: &'db dyn crate::Db) -> SignatureSymbols<'db> {
        let source = self.source(db);
        let mut symbols = SignatureSymbols::new(self);
        source.populate_signature_symbols(db, &mut symbols);
        symbols
    }

    /// Function signature
    #[salsa::tracked]
    pub fn signature(self, db: &'db dyn crate::Db) -> SymFunctionSignature<'db> {
        let scope = self.scope(db);

        let input_output = SymInputOutput {
            input_tys: self
                .source(db)
                .inputs(db)
                .iter()
                .map(|i| input_ty(db, &scope, i))
                .collect(),

            output_ty: match self.source(db).output_ty(db) {
                Some(ast_ty) => ast_ty.into_sym_in_scope(db, &scope),
                None => SymTy::unit(db),
            },
        };
        let bound_input_output = scope.into_bound_value(db, input_output);

        SymFunctionSignature::new(db, self.symbols(db).clone(), bound_input_output)
    }

    /// Returns the scope for this function; this has the function generics
    /// and parameters in scope.
    pub fn scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let symbols = self.symbols(db);
        self.scope_item(db)
            .into_scope(db)
            .with_link(Cow::Borrowed(&symbols.generics[..]))
            .with_link(Cow::Borrowed(&symbols.inputs[..]))
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

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
    scope::{Scope, ScopeItem},
    symbol::{SymGeneric, SymLocalVariable},
    ty::{Binder, SymTy, SymTyKind},
    IntoSymInScope,
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

    /// Input/output types. First level of binder is the class.
    /// Second level of binder is the function. If this is a standalone
    /// function, first binder is empty.
    pub input_output: Binder<Binder<SymInputOutput<'db>>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymInputOutput<'db> {
    pub input_tys: Vec<SymTy<'db>>,

    pub output_ty: SymTy<'db>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Update)]
pub struct SignatureSymbols<'db> {
    pub source: SignatureSource<'db>,
    pub generics: Vec<SymGeneric<'db>>,
    pub inputs: Vec<SymLocalVariable<'db>>,
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
    pub fn inputs(self, db: &'db dyn crate::Db) -> &'db [SymLocalVariable<'db>] {
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

    /// Function signature
    #[salsa::tracked]
    pub fn signature(self, db: &'db dyn crate::Db) -> SymFunctionSignature<'db> {
        let source = self.source(db);
        let mut symbols = SignatureSymbols::new(self);
        source.populate_signature_symbols(db, &mut symbols);

        let scope = self
            .scope_item(db)
            .into_scope(db)
            .ensure_binder()
            .with_link(Cow::Borrowed(&symbols));

        let input_output = SymInputOutput {
            input_tys: source
                .inputs(db)
                .iter()
                .map(|i| input_ty(db, &scope, i))
                .collect(),

            output_ty: match source.output_ty(db) {
                Some(ast_ty) => ast_ty.into_sym_in_scope(db, &scope),
                None => SymTy::unit(db),
            },
        };
        let bound_input_output = scope.into_bound(db, input_output);

        SymFunctionSignature::new(db, symbols, bound_input_output)
    }

    /// Returns the scope for this function; this has the function generics
    /// and parameters in scope.
    pub fn scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        let signature = self.signature(db);
        self.scope_item(db)
            .into_scope(db)
            .with_link(Cow::Borrowed(signature.symbols(db)))
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

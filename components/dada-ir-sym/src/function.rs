use std::borrow::Cow;

use dada_ir_ast::{
    ast::{AstFunction, AstFunctionInput, Identifier},
    diagnostic::Diagnostic,
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymClass,
    populate::PopulateSignatureSymbols,
    scope::{Scope, ScopeItem},
    symbol::{SymGeneric, SymLocalVariable},
    ty::{SymTy, SymTyKind},
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

    #[return_ref]
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
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db).id
    }

    pub fn name_span(self, db: &'db dyn crate::Db) -> Span<'db> {
        self.source(db).name(db).span
    }

    #[salsa::tracked]
    pub fn signature(self, db: &'db dyn crate::Db) -> SymFunctionSignature<'db> {
        let source = self.source(db);
        let mut symbols = SignatureSymbols::new(self);
        source.populate_signature_symbols(db, &mut symbols);
        let scope = Scope::new(db, self.scope_item(db)).with_link(Cow::Borrowed(&symbols));

        let input_tys = source
            .inputs(db)
            .iter()
            .map(|i| input_ty(db, &scope, i))
            .collect();

        let output_ty = match source.output_ty(db) {
            Some(ast_ty) => ast_ty.into_sym_in_scope(db, &scope),
            None => SymTy::unit(db),
        };

        SymFunctionSignature::new(db, symbols, input_tys, output_ty)
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

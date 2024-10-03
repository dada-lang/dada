use dada_ir_ast::{
    ast::{AstFunction, AstFunctionInput, Identifier},
    span::{Span, Spanned},
};
use dada_parser::prelude::*;
use salsa::Update;

use crate::{
    populate::PopulateSignatureSymbols,
    prelude::{IntoSymbol, ToSymbol},
    scope::{Scope, ScopeChainLink, ScopeItem},
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
    symbols: SignatureSymbols<'db>,

    #[return_ref]
    input_tys: Vec<SymTy<'db>>,

    output_ty: SymTy<'db>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Update)]
pub struct SignatureSymbols<'db> {
    pub generics: Vec<SymGeneric<'db>>,
    pub inputs: Vec<SymLocalVariable<'db>>,
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
        let mut symbols = SignatureSymbols::default();
        source.populate_signature_symbols(db, &mut symbols);
        let scope = Scope::new(db, self.scope_item(db))
            .with_link(ScopeChainLink::SignatureSymbols(&symbols));

        // Compute and store types for each input.
        for input in source.inputs(db) {
            specify_input_ty(db, &scope, input);
        }

        let input_tys = source
            .inputs(db)
            .iter()
            .map(|i| i.to_symbol(db).ty(db))
            .collect();

        let output_ty = match source.output_ty(db) {
            Some(ast_ty) => ast_ty.into_sym_in_scope(db, &scope),
            None => SymTy::unit(db),
        };

        SymFunctionSignature::new(db, symbols, input_tys, output_ty)
    }
}

fn specify_input_ty<'db>(
    db: &'db dyn crate::Db,
    scope: &Scope<'_, 'db>,
    input: &AstFunctionInput<'db>,
) -> SymTy<'db> {
    match input {
        AstFunctionInput::SelfArg(ast_self_arg) => {
            // Lookup `self` in the scope
            todo!()
        }
        AstFunctionInput::Variable(variable_decl) => {
            variable_decl.ty(db).into_sym_in_scope(db, scope)
        }
    }
}

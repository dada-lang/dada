use std::sync::Arc;

use dada_ir_ast::{
    ast::{
        AstFunction, AstFunctionInput, AstGenericArg, AstGenericDecl, AstPerm, AstPermKind, AstTy,
        AstTyKind,
    },
    span::Spanned,
};

use crate::{
    function::SignatureSymbols,
    populate,
    prelude::Symbolize,
    symbol::{SymGeneric, SymLocalVariable},
};

/// Iterate over the items in a signature (function, class, impl, etc)
/// and create the symbols for generic types and/or parameters declared within.
///
/// This is used to support Dada's "inline" declarations, e.g.
///
/// ```dada
/// fn foo(v: Vec[type T]) {}
/// ```
pub(crate) trait PopulateSignatureSymbols<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    );
}

impl<'db> PopulateSignatureSymbols<'db> for AstTy<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut crate::function::SignatureSymbols<'db>,
    ) {
        match self.kind(db) {
            AstTyKind::Perm(ast_perm, ast_ty) => {
                ast_perm.populate_signature_symbols(db, symbols);
                ast_ty.populate_signature_symbols(db, symbols);
            }
            AstTyKind::Named(_ast_path, arguments) => {
                arguments
                    .iter()
                    .flatten()
                    .for_each(|e| e.populate_signature_symbols(db, symbols));
            }
            AstTyKind::GenericDecl(ast_generic_decl) => {
                ast_generic_decl.populate_signature_symbols(db, symbols)
            }
            AstTyKind::Unknown => (),
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstPerm<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut crate::function::SignatureSymbols<'db>,
    ) {
        match self.kind(db) {
            AstPermKind::Shared(_places) => (),
            AstPermKind::Leased(_places) => (),
            AstPermKind::Given(_places) => (),
            AstPermKind::My => (),
            AstPermKind::Our => (),
            AstPermKind::Variable(_) => (),
            AstPermKind::GenericDecl(ast_generic_decl) => {
                ast_generic_decl.populate_signature_symbols(db, symbols)
            }
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstGenericArg<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut crate::function::SignatureSymbols<'db>,
    ) {
        match self {
            AstGenericArg::Ty(ast_ty) => ast_ty.populate_signature_symbols(db, symbols),
            AstGenericArg::Perm(ast_perm) => ast_perm.populate_signature_symbols(db, symbols),
            AstGenericArg::Id(_) => {}
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstGenericDecl<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        symbols.generics.push(self.symbolize(db));
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstFunction<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        self.generics(db)
            .iter()
            .flatten()
            .for_each(|g| g.populate_signature_symbols(db, symbols));

        self.inputs(db)
            .iter()
            .for_each(|i| i.populate_signature_symbols(db, symbols));
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstFunctionInput<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        match self {
            AstFunctionInput::SelfArg(ast_self_arg) => {
                symbols.inputs.push(SymLocalVariable::new(
                    db,
                    db.self_id(),
                    ast_self_arg.self_span,
                ));
            }
            AstFunctionInput::Variable(variable_decl) => {
                symbols.inputs.push(SymLocalVariable::new(
                    db,
                    variable_decl.name.id,
                    variable_decl.name.span,
                ));
                variable_decl.ty.populate_signature_symbols(db, symbols);
            }
        }
    }
}

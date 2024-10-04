use std::sync::Arc;

use dada_ir_ast::{
    ast::{
        AstClassItem, AstFunction, AstFunctionInput, AstGenericArg, AstGenericDecl, AstPerm,
        AstPermKind, AstTy, AstTyKind,
    },
    span::Spanned,
};

use crate::{
    function::SignatureSymbols,
    populate,
    prelude::IntoSymbol,
    symbol::{SymGeneric, SymLocalVariable},
    ty::{AnonymousPermSymbol, SymTy, SymTyKind},
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
            AstPermKind::Shared(Some(_))
            | AstPermKind::Leased(Some(_))
            | AstPermKind::Given(Some(_)) => (),
            AstPermKind::Shared(None) | AstPermKind::Leased(None) | AstPermKind::Given(None) => {
                symbols.generics.push(self.anonymous_perm_symbol(db));
            }
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
        symbols.generics.push(self.into_symbol(db));
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
                symbols.inputs.push(ast_self_arg.into_symbol(db));
            }
            AstFunctionInput::Variable(variable_decl) => {
                symbols.inputs.push(variable_decl.into_symbol(db));
                variable_decl.ty(db).populate_signature_symbols(db, symbols);
            }
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstClassItem<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        self.generics(db)
            .iter()
            .flatten()
            .for_each(|g| g.populate_signature_symbols(db, symbols));

        symbols
            .inputs
            .push(SymLocalVariable::new(db, db.self_id(), self.name_span(db)));
    }
}

use dada_ir_ast::ast::{
    AstAggregate, AstFunction, AstFunctionInput, AstGenericDecl, AstGenericTerm, AstPerm,
    AstPermKind, AstTy, AstTyKind,
};

use crate::{ir::functions::SignatureSymbols, ir::types::AnonymousPermSymbol, prelude::Symbol};

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
        symbols: &mut crate::ir::functions::SignatureSymbols<'db>,
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
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstPerm<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut crate::ir::functions::SignatureSymbols<'db>,
    ) {
        match self.kind(db) {
            AstPermKind::Shared(Some(_))
            | AstPermKind::Leased(Some(_))
            | AstPermKind::Given(Some(_)) => (),

            AstPermKind::Default
            | AstPermKind::Shared(None)
            | AstPermKind::Leased(None)
            | AstPermKind::Given(None) => {
                symbols
                    .generic_variables
                    .push(self.anonymous_perm_symbol(db));
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

impl<'db> PopulateSignatureSymbols<'db> for AstGenericTerm<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut crate::ir::functions::SignatureSymbols<'db>,
    ) {
        match self {
            AstGenericTerm::Ty(ast_ty) => ast_ty.populate_signature_symbols(db, symbols),
            AstGenericTerm::Perm(ast_perm) => ast_perm.populate_signature_symbols(db, symbols),
            AstGenericTerm::Id(_) => {}
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstGenericDecl<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        symbols.generic_variables.push(self.symbol(db));
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
                ast_self_arg
                    .perm(db)
                    .populate_signature_symbols(db, symbols);
                symbols.input_variables.push(ast_self_arg.symbol(db));
            }
            AstFunctionInput::Variable(variable_decl) => {
                symbols.input_variables.push(variable_decl.symbol(db));
                variable_decl.ty(db).populate_signature_symbols(db, symbols);
            }
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstAggregate<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        self.generics(db)
            .iter()
            .flatten()
            .for_each(|g| g.populate_signature_symbols(db, symbols));
    }
}

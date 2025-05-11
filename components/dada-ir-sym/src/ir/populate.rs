use dada_ir_ast::ast::{
    AstAggregate, AstFunction, AstFunctionInput, AstGenericDecl, AstGenericTerm, AstPerm,
    AstPermKind, AstSelfArg, AstTy, AstTyKind, VariableDecl,
};

use crate::{
    check::scope::{ResolveToSym, Scope},
    ir::{functions::SignatureSymbols, types::AnonymousPermSymbol},
    prelude::Symbol,
};

use super::{classes::SymAggregateStyle, functions::SymFunctionSource};

/// Iterate over the items in a signature (function, class, impl, etc)
/// and create the symbols for generic types and/or parameters declared within.
/// It is used to support Dada's "inline" declarations, e.g.
///
/// ```dada
/// fn foo(v: Vec[type T]) {}
/// ```
///
/// This method is only concerned with explicit declarations, not defaulted ones,
/// which are handled by [`PopulateDefaultSymbols`].
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
            AstPermKind::Referenced(Some(_))
            | AstPermKind::Mutable(Some(_))
            | AstPermKind::Given(Some(_)) => (),

            AstPermKind::Referenced(None)
            | AstPermKind::Mutable(None)
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
                ast_self_arg.populate_signature_symbols(db, symbols);
                symbols.input_variables.push(ast_self_arg.symbol(db));
            }
            AstFunctionInput::Variable(variable_decl) => {
                variable_decl.populate_signature_symbols(db, symbols);
                symbols.input_variables.push(variable_decl.symbol(db));
            }
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for AstSelfArg<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        if let Some(perm) = self.perm(db) {
            perm.populate_signature_symbols(db, symbols);
        } else {
            // See `PopulateDefaultSymbols` below.
        }
    }
}

impl<'db> PopulateSignatureSymbols<'db> for VariableDecl<'db> {
    fn populate_signature_symbols(
        &self,
        db: &'db dyn crate::Db,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        if let Some(perm) = self.perm(db) {
            perm.populate_signature_symbols(db, symbols);
        } else {
            // See `PopulateDefaultSymbols` below.
        }

        self.base_ty(db).populate_signature_symbols(db, symbols);
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

impl<'db> PopulateSignatureSymbols<'db> for SymFunctionSource<'db> {
    fn populate_signature_symbols(
        &self,
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

/// In a few specific places, we add in *default* permissions.
/// These are always an anonymous symbol, so they don't impact
/// name resolution (this is important to avoid cycles, see the note below).
///
/// This method pushes those default permissions, if any,
/// into `symbols`.
///
/// # Examples
///
/// * Given `class C { fn foo(self) }`, the `self` has a default permission
/// * Given `fn foo(x: String)`, the variable `x` has a default permission
///
/// Default permissions are only needed for classes and when explicit permissions are not provided
///
/// * Given `struct C { fn foo(self) }`, the `self` does NOT have a default permission
/// * Given `fn foo(x: u32)`, the variable `x` does NOT have a default permission
/// * Given `fn foo(x: my String)`, the variable `x` does NOT have a default permission
/// * Given `fn foo(x: type T)`, the variable `x` does NOT have a default permission
///
/// # Note on cycles
///
/// Given `x: Foo`, we need to determine if `Foo` is a struct or a class
/// to decide whether to give it a default symbol. This requires a name
/// resolution scope. But creating name resolution scopes required knowing
/// the symbols in scope, and default permissions would be in scope.
/// This is a "false cycle" because default permissions are anonymous.
/// Nonetheless, this is why we separate out populating default symbols
/// from the primary [`PopulateSignatureSymbols`] function.
pub(crate) trait PopulateDefaultSymbols<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    );
}

impl<'db> PopulateDefaultSymbols<'db> for AstSelfArg<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        if self_arg_requires_default_perm(db, *self, scope) {
            symbols
                .generic_variables
                .push(self.anonymous_perm_symbol(db));
        }
    }
}

/// Returns true if a self arg requires a default permission.
/// See [`PopulateDefaultSymbols`] trait for examples.
pub(crate) fn self_arg_requires_default_perm<'db>(
    db: &'db dyn crate::Db,
    decl: AstSelfArg<'db>,
    scope: &Scope<'_, 'db>,
) -> bool {
    if decl.perm(db).is_some() {
        return false;
    }

    match scope.aggregate().map(|a| a.style(db)) {
        None | Some(SymAggregateStyle::Struct) => {
            // Methods on structs don't need a default permission.
            false
        }

        Some(SymAggregateStyle::Class) => {
            // Methods on classes will default to any permission.
            true
        }
    }
}

impl<'db> PopulateDefaultSymbols<'db> for VariableDecl<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        if variable_decl_requires_default_perm(db, *self, scope) {
            symbols
                .generic_variables
                .push(self.anonymous_perm_symbol(db));
        }
    }
}

/// Returns true if a variable declaration requires a default permission.
/// See [`PopulateDefaultSymbols`] trait for examples.
pub(crate) fn variable_decl_requires_default_perm<'db>(
    db: &'db dyn crate::Db,
    decl: VariableDecl<'db>,
    scope: &Scope<'_, 'db>,
) -> bool {
    if decl.perm(db).is_some() {
        return false;
    }

    match decl.base_ty(db).kind(db) {
        AstTyKind::Perm(..) => {
            panic!("should not have an explicit permission if VariableDecl's `perm` field is None")
        }
        AstTyKind::Named(path, _) => {
            if let Ok(sym) = path.resolve_to_sym(db, scope) {
                match sym.style(db) {
                    Some(SymAggregateStyle::Struct) | None => false,
                    Some(SymAggregateStyle::Class) => true,
                }
            } else {
                false
            }
        }
        AstTyKind::GenericDecl(..) => {
            // No default symbol in this case.
            false
        }
    }
}

impl<'db> PopulateDefaultSymbols<'db> for SymFunctionSource<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        match self {
            Self::Function(ast_function) => {
                ast_function.populate_default_symbols(db, scope, symbols)
            }
            Self::Constructor(..) => {
                self.inputs(db)
                    .iter()
                    .for_each(|i| i.populate_default_symbols(db, scope, symbols));
            }
        }
    }
}

impl<'db> PopulateDefaultSymbols<'db> for AstFunction<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        for input in self.inputs(db) {
            input.populate_default_symbols(db, scope, symbols);
        }
    }
}

impl<'db> PopulateDefaultSymbols<'db> for AstFunctionInput<'db> {
    fn populate_default_symbols(
        &self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
        symbols: &mut SignatureSymbols<'db>,
    ) {
        match self {
            AstFunctionInput::SelfArg(ast_self_arg) => {
                ast_self_arg.populate_default_symbols(db, scope, symbols)
            }
            AstFunctionInput::Variable(variable_decl) => {
                variable_decl.populate_default_symbols(db, scope, symbols)
            }
        }
    }
}

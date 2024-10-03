use dada_ir_ast::{
    ast::{AstFieldDecl, AstFunctionInput, AstSelfArg, Identifier, VariableDecl},
    span::Span,
};
use salsa::Update;

use crate::{
    prelude::{IntoSymbol, ToSymbol},
    ty::SymTy,
};

#[salsa::tracked]
pub struct SymLocalVariable<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
}

impl<'db> SymLocalVariable<'db> {
    /// Returns the type of this local variable.
    ///
    /// This is a "lazy field" populated by specifying
    /// the value of the tracked function [`local_var_ty`][].
    ///
    /// # Panics
    ///
    /// Panics if `specify` has not yet been invoked on [`local_var_ty`][].
    pub fn ty(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        local_var_ty(db, self)
    }
}

/// See [`SymLocalVariable::ty`][]
#[salsa::tracked(specify)]
pub fn local_var_ty<'db>(_db: &'db dyn crate::Db, var: SymLocalVariable<'db>) -> SymTy<'db> {
    // FIXME: This should be a salsa feature
    panic!("Ty for `{var:?}` not yet specified")
}

#[salsa::tracked]
pub struct SymField<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
    pub source: AstFieldDecl<'db>,
}

/// Declaration of a generic parameter.
#[salsa::tracked]
pub struct SymGeneric<'db> {
    pub kind: SymGenericKind,
    pub name: Option<Identifier<'db>>,
    pub span: Span<'db>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Update, Debug)]
pub enum SymGenericKind {
    Type,
    Perm,
}

impl<'db> ToSymbol<'db> for AstFunctionInput<'db> {
    type Symbolic = SymLocalVariable<'db>;

    fn to_symbol(&self, db: &'db dyn crate::Db) -> SymLocalVariable<'db> {
        match self {
            AstFunctionInput::SelfArg(ast_self_arg) => ast_self_arg.into_symbol(db),
            AstFunctionInput::Variable(variable_decl) => variable_decl.into_symbol(db),
        }
    }
}

#[salsa::tracked]
impl<'db> IntoSymbol<'db> for VariableDecl<'db> {
    type Symbolic = SymLocalVariable<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymLocalVariable<'db> {
        SymLocalVariable::new(db, self.name(db).id, self.name(db).span)
    }
}

#[salsa::tracked]
impl<'db> IntoSymbol<'db> for AstSelfArg<'db> {
    type Symbolic = SymLocalVariable<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymLocalVariable<'db> {
        SymLocalVariable::new(db, db.self_id(), self.self_span(db))
    }
}

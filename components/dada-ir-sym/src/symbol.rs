use dada_ir_ast::{
    ast::{AstFieldDecl, Identifier},
    span::Span,
};
use salsa::Update;

use crate::ty::SymTy;

#[salsa::tracked]
pub struct SymLocalVariable<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
}

impl<'db> SymLocalVariable<'db> {
    pub fn ty(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        local_var_ty(db, self)
    }
}

#[salsa::tracked(specify)]
pub fn local_var_ty<'db>(_db: &'db dyn crate::Db, var: SymLocalVariable<'db>) -> SymTy<'db> {
    // FIXME: This should be a salsa feature
    panic!("Type for `{var:?}` not yet specified")
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

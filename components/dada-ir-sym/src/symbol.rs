use dada_ir_ast::{
    ast::{AstFieldDecl, Identifier},
    span::Span,
};

use crate::ty::SymGenericKind;

#[salsa::tracked]
pub struct SymLocalVariable<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
}

#[salsa::tracked]
pub struct SymField<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
    pub source: AstFieldDecl<'db>,
}

#[salsa::tracked]
pub struct SymGeneric<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
    pub kind: SymGenericKind,
}

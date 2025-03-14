use dada_util::{FromImpls, SalsaSerialize};
use salsa::Update;
use serde::Serialize;

use crate::{
    ast::AstVisibility,
    span::{Span, Spanned},
};

use super::{AstFunction, VariableDecl};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, FromImpls, Serialize)]
pub enum AstMember<'db> {
    Field(AstFieldDecl<'db>),
    Function(AstFunction<'db>),
}

impl<'db> Spanned<'db> for AstMember<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstMember::Field(field) => field.span(db),
            AstMember::Function(function) => function.span(db),
        }
    }
}

#[derive(SalsaSerialize)]
#[salsa::tracked]
pub struct AstFieldDecl<'db> {
    pub span: Span<'db>,
    pub visibility: Option<AstVisibility<'db>>,
    pub variable: VariableDecl<'db>,
}

impl<'db> Spanned<'db> for AstFieldDecl<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        AstFieldDecl::span(*self, db)
    }
}

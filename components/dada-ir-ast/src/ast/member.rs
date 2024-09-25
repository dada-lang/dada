use salsa::Update;

use crate::span::{Span, Spanned};

use super::{AstFunction, VariableDecl};

add_from_impls! {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
    pub enum AstMember<'db> {
        Field(AstFieldDecl<'db>),
        Function(AstFunction<'db>),
    }
}

impl<'db> Spanned<'db> for AstMember<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            AstMember::Field(field) => field.span(db),
            AstMember::Function(function) => function.span(db),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstFieldDecl<'db> {
    pub span: Span<'db>,
    pub visibility: Option<AstVisibility<'db>>,
    pub variable: VariableDecl<'db>,
}

impl<'db> Spanned<'db> for AstFieldDecl<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstVisibility<'db> {
    pub span: Span<'db>,
    pub kind: VisibilityKind,
}

impl<'db> Spanned<'db> for AstVisibility<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum VisibilityKind {
    Export,
    Pub,
}

use salsa::{DebugWithDb, Update};

use crate::span::{Span, Spanned};

use super::{Function, VariableDecl};

add_from_impls! {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
    pub enum Member<'db> {
        Field(FieldDecl<'db>),
        Function(Function<'db>),
    }
}

impl<'db> Spanned<'db> for Member<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            Member::Field(field) => field.span(db),
            Member::Function(function) => function.span(db),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct FieldDecl<'db> {
    pub span: Span<'db>,
    pub visibility: Option<Visibility<'db>>,
    pub variable: VariableDecl<'db>,
}

impl<'db> Spanned<'db> for FieldDecl<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub struct Visibility<'db> {
    pub span: Span<'db>,
    pub kind: VisibilityKind,
}

impl<'db> Spanned<'db> for Visibility<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        self.span
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum VisibilityKind {
    Export,
    Pub,
}

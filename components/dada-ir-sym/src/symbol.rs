use dada_ir_ast::{
    ast::{AstFunctionInput, AstSelfArg, Identifier, VariableDecl},
    span::{Span, Spanned},
};
use salsa::Update;

use crate::prelude::{IntoSymbol, ToSymbol};

#[salsa::tracked]
pub struct SymLocalVariable<'db> {
    pub name: Identifier<'db>,
    pub name_span: Span<'db>,
}

impl<'db> Spanned<'db> for SymLocalVariable<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.name_span(db)
    }
}

/// Declaration of a generic parameter.
#[salsa::tracked]
pub struct SymGeneric<'db> {
    pub kind: SymGenericKind,
    pub name: Option<Identifier<'db>>,
    pub span: Span<'db>,
}

impl<'db> Spanned<'db> for SymGeneric<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        SymGeneric::span(*self, db)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
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

impl std::fmt::Display for SymGenericKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "type"),
            Self::Perm => write!(f, "perm"),
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

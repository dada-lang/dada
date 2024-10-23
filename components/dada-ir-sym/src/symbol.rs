use dada_ir_ast::{
    ast::{AstFunctionInput, AstSelfArg, Identifier, VariableDecl},
    span::{Span, Spanned},
};
use salsa::Update;

use crate::{
    prelude::{IntoSymbol, ToSymbol},
    scope::Scope,
    ty::{FromVar, SymGenericTerm},
};

/// Symbol for a generic parameter or local variable.
#[salsa::tracked]
pub struct SymVariable<'db> {
    pub kind: SymGenericKind,
    pub name: Option<Identifier<'db>>,
    pub span: Span<'db>,
}

impl<'db> SymVariable<'db> {
    /// New symbol for a local variable
    pub fn new_local(db: &'db dyn crate::Db, id: Identifier<'db>, span: Span<'db>) -> Self {
        Self::new(db, SymGenericKind::Place, Some(id), span)
    }

    pub fn into_generic_term(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> SymGenericTerm<'db> {
        let var = scope.resolve_generic_sym(db, self);
        SymGenericTerm::var(db, self.kind(db), var)
    }
}

impl<'db> HasKind<'db> for SymVariable<'db> {
    fn has_kind(&self, db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        self.kind(db) == kind
    }
}

impl<'db> Spanned<'db> for SymVariable<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        SymVariable::span(*self, db)
    }
}

impl std::fmt::Display for SymVariable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| match self.name(db) {
            Some(name) => write!(f, "`{}`", name),
            None => write!(f, "generic `{kind}`", kind = self.kind(db)),
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymGenericKind {
    Type,
    Perm,
    Place,
}

/// Test if `self` can be said to have the given kind (i.e., is it a type? a permission?).
///
/// Note that when errors occur, this may return true for multiple kinds.
pub trait HasKind<'db> {
    fn has_kind(&self, db: &'db dyn crate::Db, kind: SymGenericKind) -> bool;
}

impl<'db> ToSymbol<'db> for AstFunctionInput<'db> {
    type Symbolic = SymVariable<'db>;

    fn to_symbol(&self, db: &'db dyn crate::Db) -> SymVariable<'db> {
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
            Self::Place => write!(f, "place"),
        }
    }
}

#[salsa::tracked]
impl<'db> IntoSymbol<'db> for VariableDecl<'db> {
    type Symbolic = SymVariable<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        SymVariable::new(
            db,
            SymGenericKind::Place,
            Some(self.name(db).id),
            self.name(db).span,
        )
    }
}

#[salsa::tracked]
impl<'db> IntoSymbol<'db> for AstSelfArg<'db> {
    type Symbolic = SymVariable<'db>;

    #[salsa::tracked]
    fn into_symbol(self, db: &'db dyn crate::Db) -> SymVariable<'db> {
        SymVariable::new(
            db,
            SymGenericKind::Place,
            Some(db.self_id()),
            self.self_span(db),
        )
    }
}

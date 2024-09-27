use dada_ir_ast::ast::{AstGenericDecl, AstGenericKind};

use crate::{
    prelude::Symbolize,
    ty::{SymGenericDecl, SymGenericKind},
};

#[salsa::tracked]
impl<'db> Symbolize<'db> for AstGenericDecl<'db> {
    type Symbolic = SymGenericDecl<'db>;

    #[salsa::tracked]
    fn symbolize(self, db: &'db dyn crate::Db) -> SymGenericDecl<'db> {
        SymGenericDecl::new(
            db,
            self.kind(db).symbolize(db),
            self.decl(db).name.id,
            self.decl(db).name.span,
        )
    }
}

impl<'db> Symbolize<'db> for AstGenericKind<'db> {
    type Symbolic = SymGenericKind;

    fn symbolize(self, _db: &'db dyn crate::Db) -> Self::Symbolic {
        match self {
            AstGenericKind::Type(_) => SymGenericKind::Type,
            AstGenericKind::Perm(_) => SymGenericKind::Perm,
        }
    }
}

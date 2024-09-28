use dada_ir_ast::{
    ast::{AstGenericDecl, AstGenericKind},
    span::Spanned,
};

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
            self.name(db).map(|n| n.id),
            self.span(db),
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

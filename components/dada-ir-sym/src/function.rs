use dada_ir_ast::{
    ast::{AstFunction, AstItem, AstModule, Identifier},
    span::{Span, Spanned},
};
use dada_parser::prelude::*;

use crate::{prelude::Symbolize, scope::ScopeItem, symbol::SymGeneric, ty::SymTy};

#[salsa::tracked]
pub struct SymFunction<'db> {
    enclosing_item: ScopeItem<'db>,
    source: AstFunction<'db>,
}

impl<'db> Spanned<'db> for SymFunction<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).name(db).span
    }
}

#[salsa::tracked]
pub struct SymFunctionSignature<'db> {
    source: SymFunction<'db>,

    pub generics: Vec<SymGeneric<'db>>,
    pub inputs: Vec<SymTy<'db>>,
    pub output: SymTy<'db>,
}

#[salsa::tracked]
impl<'db> SymFunction<'db> {
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db).id
    }

    pub fn name_span(self, db: &'db dyn crate::Db) -> Span<'db> {
        self.source(db).name(db).span
    }

    #[salsa::tracked]
    pub fn signature(self, db: &'db dyn crate::Db) -> SymFunctionSignature<'db> {
        let source = self.source(db);

        let mut generics = vec![];
        if let Some(declared_generics) = source.generics(db) {
            for generic_decl in &declared_generics.values {
                generics.push(generic_decl.symbolize(db));
            }
        }

        todo!()
    }
}

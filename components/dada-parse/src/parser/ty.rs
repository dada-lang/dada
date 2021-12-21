use crate::parser::Parser;

use dada_ir::ty::Ty;

impl<'db> Parser<'db> {
    pub(crate) fn parse_ty(&mut self) -> Option<Ty> {
        None
    }
}

use dada_ir_sym::{symbol::SymGenericKind, ty::SymGenericTerm};

use crate::universe::Universe;

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,
    universe: Universe,
    lower_bounds: Vec<SymGenericTerm<'db>>,
    upper_bounds: Vec<SymGenericTerm<'db>>,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe) -> Self {
        Self {
            kind,
            universe,
            lower_bounds: vec![],
            upper_bounds: vec![],
        }
    }

    pub fn kind(&self) -> SymGenericKind {
        self.kind
    }

    pub fn lower_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.lower_bounds
    }

    pub fn upper_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.upper_bounds
    }
}

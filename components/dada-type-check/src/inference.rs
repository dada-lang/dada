use dada_ir_sym::{symbol::SymGenericKind, ty::SymGenericTerm};

use crate::{bound::Bound, universe::Universe};

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

    pub fn push_bound(&mut self, bound: Bound<SymGenericTerm<'db>>) {
        assert!(bound.has_kind(self.kind));
        match bound {
            Bound::LowerBound(term) => self.lower_bounds.push(term),
            Bound::UpperBound(term) => self.upper_bounds.push(term),
        }
    }

    pub fn upper_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.upper_bounds
    }
    pub fn lower_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.lower_bounds
    }
}

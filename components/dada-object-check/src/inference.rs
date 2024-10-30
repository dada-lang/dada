use dada_ir_ast::span::Span;
use dada_ir_sym::symbol::{HasKind, SymGenericKind};

use crate::{bound::Bound, object_ir::ObjectGenericTerm, universe::Universe};

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,
    universe: Universe,
    span: Span<'db>,

    lower_bounds: Vec<ObjectGenericTerm<'db>>,
    upper_bounds: Vec<ObjectGenericTerm<'db>>,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe, span: Span<'db>) -> Self {
        Self {
            kind,
            universe,
            span,
            lower_bounds: vec![],
            upper_bounds: vec![],
        }
    }

    pub fn kind(&self) -> SymGenericKind {
        self.kind
    }

    pub fn push_bound(&mut self, db: &'db dyn crate::Db, bound: Bound<ObjectGenericTerm<'db>>) {
        assert!(bound.has_kind(db, self.kind));
        match bound {
            Bound::LowerBound(term) => self.lower_bounds.push(term),
            Bound::UpperBound(term) => self.upper_bounds.push(term),
        }
    }

    pub fn lower_bounds(&self) -> &[ObjectGenericTerm<'db>] {
        &self.lower_bounds
    }

    pub fn upper_bounds(&self) -> &[ObjectGenericTerm<'db>] {
        &self.upper_bounds
    }
}

use dada_ir_ast::span::Span;
use dada_util::vecset::VecSet;

use crate::{
    bound::Direction, ir::types::SymGenericKind, ir::types::SymGenericTerm, universe::Universe,
};

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,

    #[expect(dead_code)]
    universe: Universe,

    span: Span<'db>,

    lower_bounds: VecSet<SymGenericTerm<'db>>,
    upper_bounds: VecSet<SymGenericTerm<'db>>,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe, span: Span<'db>) -> Self {
        Self {
            kind,
            universe,
            span,
            lower_bounds: Default::default(),
            upper_bounds: Default::default(),
        }
    }

    pub fn span(&self) -> Span<'db> {
        self.span
    }

    pub fn insert_bound(
        &mut self,
        db: &'db dyn crate::Db,
        direction: Direction,
        term: SymGenericTerm<'db>,
    ) -> bool {
        assert!(term.has_kind(db, self.kind));
        match direction {
            Direction::LowerBoundedBy => self.lower_bounds.insert(term),
            Direction::UpperBoundedBy => self.upper_bounds.insert(term),
        }
    }

    pub fn lower_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.lower_bounds
    }

    pub fn upper_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.upper_bounds
    }
}

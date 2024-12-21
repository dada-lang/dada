use dada_ir_ast::span::Span;
use dada_util::vecset::VecSet;

use crate::{ir::types::SymGenericKind, ir::types::SymGenericTerm, ir::universe::Universe};

pub struct InferenceVarData<'db> {
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

    pub fn kind(&self) -> SymGenericKind {
        self.kind
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
    LowerBoundedBy,
    UpperBoundedBy,
}

impl Direction {
    pub fn reverse(self) -> Self {
        match self {
            Direction::LowerBoundedBy => Direction::UpperBoundedBy,
            Direction::UpperBoundedBy => Direction::LowerBoundedBy,
        }
    }

    pub fn infer_var_bounds<'i, 'db>(
        self,
        data: &'i InferenceVarData<'db>,
    ) -> &'i [SymGenericTerm<'db>] {
        match self {
            Direction::LowerBoundedBy => data.lower_bounds(),
            Direction::UpperBoundedBy => data.upper_bounds(),
        }
    }
}

use dada_ir_sym::{
    symbol::{HasKind, SymGenericKind},
    ty::SymGenericTerm,
};
use dada_util::vecset::VecSet;

use crate::{bound::Bound, object_ir::ObjectGenericTerm, universe::Universe};

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,
    universe: Universe,

    lower_bounds: Vec<SymGenericTerm<'db>>,
    upper_bounds: Vec<SymGenericTerm<'db>>,

    lower_object_bounds: VecSet<ObjectGenericTerm<'db>>,
    upper_object_bounds: VecSet<ObjectGenericTerm<'db>>,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe) -> Self {
        Self {
            kind,
            universe,
            lower_bounds: vec![],
            upper_bounds: vec![],
            lower_object_bounds: Default::default(),
            upper_object_bounds: Default::default(),
        }
    }

    pub fn kind(&self) -> SymGenericKind {
        self.kind
    }

    pub fn push_bound(&mut self, db: &'db dyn crate::Db, bound: Bound<SymGenericTerm<'db>>) {
        assert!(bound.has_kind(db, self.kind));
        match bound {
            Bound::LowerBound(term) => self.lower_bounds.push(term),
            Bound::UpperBound(term) => self.upper_bounds.push(term),
        }
    }

    pub fn lower_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.lower_bounds
    }

    pub fn upper_bounds(&self) -> &[SymGenericTerm<'db>] {
        &self.upper_bounds
    }

    pub fn lower_object_bounds(&self) -> &[ObjectGenericTerm<'db>] {
        &self.lower_object_bounds
    }

    pub fn upper_object_bounds(&self) -> &[ObjectGenericTerm<'db>] {
        &self.upper_object_bounds
    }
}

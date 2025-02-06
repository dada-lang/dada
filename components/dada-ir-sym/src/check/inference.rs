use dada_ir_ast::span::Span;
use dada_util::vecset::VecSet;

use crate::{
    check::universe::Universe,
    ir::{red_terms::RedTerm, types::SymGenericKind},
};

use super::predicates::Predicate;

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,

    #[expect(dead_code)]
    universe: Universe,

    span: Span<'db>,

    is: VecSet<Predicate>,
    isnt: VecSet<Predicate>,
    lower_bound: Option<RedTerm<'db>>,
    upper_bound: Option<RedTerm<'db>>,

    modifications: u32,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe, span: Span<'db>) -> Self {
        Self {
            kind,
            universe,
            span,
            is: Default::default(),
            isnt: Default::default(),
            lower_bound: None,
            upper_bound: None,
            modifications: 0,
        }
    }

    /// Returns the span of code which triggered the inference variable to be created.
    pub fn span(&self) -> Span<'db> {
        self.span
    }

    /// Returns the kind of the inference variable.
    pub fn kind(&self) -> SymGenericKind {
        self.kind
    }

    /// Returns the set of predicates that must be true.
    /// Always disjoint from [`Self::isnt`].
    pub fn is(&self) -> &VecSet<Predicate> {
        &self.is
    }

    /// Returns the set of predicates that must be false.
    /// Always disjoint from [`Self::is`].
    pub fn isnt(&self) -> &VecSet<Predicate> {
        &self.isnt
    }

    /// Returns the lower bound.
    pub fn lower_bound(&self) -> Option<RedTerm<'db>> {
        self.lower_bound
    }

    /// Returns the upper bound.
    pub fn upper_bound(&self) -> Option<RedTerm<'db>> {
        self.upper_bound
    }

    /// Insert a predicate into the `is` set.
    /// Returns `true` if the predicate was not already in the set.
    /// Low-level method invoked by runtime only.
    pub fn insert_is_predicate(&mut self, predicate: Predicate) -> bool {
        assert!(!self.isnt.contains(&predicate));
        if self.is.insert(predicate) {
            self.modifications += 1;
            true
        } else {
            false
        }
    }

    /// Insert a predicate into the `isnt` set.
    /// Returns `true` if the predicate was not already in the set.
    /// Low-level method invoked by runtime only.
    pub fn insert_isnt_predicate(&mut self, predicate: Predicate) -> bool {
        assert!(!self.is.contains(&predicate));
        if self.isnt.insert(predicate) {
            self.modifications += 1;
            true
        } else {
            false
        }
    }

    /// Set the lower bound.
    /// Low-level method invoked by runtime only.
    pub fn set_lower_bound(&mut self, lower_bound: RedTerm<'db>) {
        self.lower_bound = Some(lower_bound);
        self.modifications += 1;
    }

    /// Set the upper bound.
    /// Low-level method invoked by runtime only.
    pub fn set_upper_bound(&mut self, upper_bound: RedTerm<'db>) {
        self.upper_bound = Some(upper_bound);
        self.modifications += 1;
    }

    /// Returns the number of modifications to this inference variable.
    pub fn modifications(&self) -> u32 {
        self.modifications
    }
}

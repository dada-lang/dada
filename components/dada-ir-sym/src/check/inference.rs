use dada_ir_ast::span::Span;
use dada_util::vecset::VecSet;

use crate::{
    check::universe::Universe,
    ir::{indices::InferVarIndex, red_terms::RedTerm, types::SymGenericKind},
};

use super::predicates::Predicate;

pub(crate) struct InferenceVarData<'db> {
    kind: SymGenericKind,

    #[expect(dead_code)]
    universe: Universe,

    span: Span<'db>,

    /// If the element for a given predicate is `Some`, then the predicate is known to be true
    /// due to code at the given span. If the element is `None`, then it is not known that the
    /// predicate is true (but it still could be, depending on what value we ultimately infer).
    ///
    /// See also the `isnt` field.
    is: [Option<Span<'db>>; Predicate::LEN],

    /// If the element for a given predicate is `Some`, then the predicate is NOT known to be true
    /// due to code at the given span.
    ///
    /// This is a subtle distinction. Knowing that a variable `isnt (known to be) copy` doesn't
    /// imply that it is `is (known to be) move`. It means "you will never be able to prove this is copy".
    isnt: [Option<Span<'db>>; Predicate::LEN],

    lower_bound: Option<RedTerm<'db>>,
    upper_bound: Option<RedTerm<'db>>,

    lower_bound_vars: Vec<InferVarIndex>,
    upper_bound_vars: Vec<InferVarIndex>,

    modifications: u32,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe, span: Span<'db>) -> Self {
        Self {
            kind,
            universe,
            span,
            is: [None; Predicate::LEN],
            isnt: [None; Predicate::LEN],
            lower_bound: None,
            upper_bound: None,
            lower_bound_vars: Vec::new(),
            upper_bound_vars: Vec::new(),
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

    /// Returns `Some(s)` if the predicate is known to be true (where `s` is the span of code
    /// which required the predicate to be true).
    pub fn is_known_to_be(&self, predicate: Predicate) -> Option<Span<'db>> {
        self.is[predicate.index()]
    }

    /// Returns `Some(s)` if the predicate is not known to be true (where `s` is the span of code
    /// which required the predicate to not be known to be true).
    ///
    /// This is different from being known to be false. It means we know we won't be able to know.
    /// Can occur with generics etc.
    pub fn isnt_known_to_be(&self, predicate: Predicate) -> Option<Span<'db>> {
        self.isnt[predicate.index()]
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
    ///
    /// # Panics
    ///
    /// * If the inference variable is required to satisfy a contradictory predicate.
    pub fn require_is(&mut self, predicate: Predicate, span: Span<'db>) -> bool {
        assert!(self.is_known_to_be(predicate.invert()).is_none());
        assert!(self.isnt_known_to_be(predicate).is_none());
        if self.is_known_to_be(predicate).is_none() {
            self.is[predicate.index()] = Some(span);
            self.modifications += 1;
            true
        } else {
            false
        }
    }

    /// Insert a predicate into the `isnt` set.
    /// Returns `true` if the predicate was not already in the set.
    /// Low-level method invoked by runtime only.
    ///
    /// # Panics
    ///
    /// * If the inference variable is required to satisfy a contradictory predicate.
    pub fn require_isnt(&mut self, predicate: Predicate, span: Span<'db>) -> bool {
        assert!(self.is_known_to_be(predicate).is_none());
        if self.isnt_known_to_be(predicate).is_none() {
            self.isnt[predicate.index()] = Some(span);
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

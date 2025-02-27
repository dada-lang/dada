use std::sync::Arc;

use dada_ir_ast::{diagnostic::Errors, span::Span};
use dada_util::vecset::VecSet;

use crate::{
    check::universe::Universe,
    ir::{indices::InferVarIndex, red_terms::RedTerm, types::SymGenericKind},
};

use super::{
    chains::{Chain, RedTy},
    env::Env,
    predicates::Predicate,
    report::OrElse,
};

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
    is: [Option<Arc<dyn OrElse<'db> + 'db>>; Predicate::LEN],

    /// If the element for a given predicate is `Some`, then the predicate is NOT known to be true
    /// due to code at the given span.
    ///
    /// This is a subtle distinction. Knowing that a variable `isnt (known to be) copy` doesn't
    /// imply that it is `is (known to be) move`. It means "you will never be able to prove this is copy".
    isnt: [Option<Arc<dyn OrElse<'db> + 'db>>; Predicate::LEN],

    lower_chains: VecSet<Chain<'db>>,
    upper_chains: VecSet<Chain<'db>>,

    lower_red_ty: Option<RedTy<'db>>,
    upper_red_ty: Option<RedTy<'db>>,
}

impl<'db> InferenceVarData<'db> {
    pub fn new(kind: SymGenericKind, universe: Universe, span: Span<'db>) -> Self {
        Self {
            kind,
            universe,
            span,
            is: [None, None, None, None],
            isnt: [None, None, None, None],
            lower_chains: VecSet::new(),
            upper_chains: VecSet::new(),
            lower_red_ty: None,
            upper_red_ty: None,
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
    pub fn is_known_to_provably_be(
        &self,
        predicate: Predicate,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.is[predicate.index()].clone()
    }

    /// Returns `Some(s)` if the predicate is not known to be true (where `s` is the span of code
    /// which required the predicate to not be known to be true).
    ///
    /// This is different from being known to be false. It means we know we won't be able to know.
    /// Can occur with generics etc.
    pub fn is_known_not_to_provably_be(
        &self,
        predicate: Predicate,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.isnt[predicate.index()].clone()
    }

    /// Returns the lower bound.
    pub fn lower_chains(&self) -> &VecSet<Chain<'db>> {
        &self.lower_chains
    }

    /// Returns the upper bound.
    pub fn upper_chains(&self) -> &VecSet<Chain<'db>> {
        &self.upper_chains
    }

    /// Insert a predicate into the `is` set and its invert into the `isnt` set.
    /// Returns `None` if these are not new requirements.
    /// Otherwise, returns `Some(o)` where `o` is the Arc-ified version of `or_else`.
    /// Low-level method invoked by runtime only.
    ///
    /// # Panics
    ///
    /// * If the inference variable is required to satisfy a contradictory predicate.
    pub fn require_is(
        &mut self,
        predicate: Predicate,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        let predicate_invert = predicate.invert();

        let predicate_is = self.is_known_to_provably_be(predicate).is_some();
        let predicate_isnt = self.is_known_not_to_provably_be(predicate).is_some();
        let inverted_is = self.is_known_to_provably_be(predicate_invert).is_some();
        let inverted_isnt = self.is_known_not_to_provably_be(predicate_invert).is_some();

        // Check that we haven't been given contradictory constraints.
        assert!(
            !predicate_isnt,
            "require_is: {predicate} already required to be isnt"
        );
        assert!(
            !inverted_is,
            "require_is: {predicate_invert} already required to be is"
        );

        // If these constraints are alreayd recorded, just return.
        if predicate_is && inverted_isnt {
            return None;
        }

        // Otherwise record.
        let or_else = or_else.to_arc();
        if !predicate_is {
            self.is[predicate.index()] = Some(or_else.clone());
        }
        if !inverted_isnt {
            self.isnt[predicate_invert.index()] = Some(or_else.clone());
        }
        Some(or_else)
    }

    /// Insert a predicate into the `isnt` set.
    /// Returns `true` if the predicate was not already in the set.
    /// Low-level method invoked by runtime only.
    ///
    /// # Panics
    ///
    /// * If the inference variable is required to satisfy a contradictory predicate.
    pub fn require_isnt(
        &mut self,
        predicate: Predicate,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        assert!(self.is_known_to_provably_be(predicate).is_none());
        if self.is_known_not_to_provably_be(predicate).is_none() {
            let or_else = or_else.to_arc();
            self.isnt[predicate.index()] = Some(or_else.clone());
            Some(or_else)
        } else {
            None
        }
    }
}

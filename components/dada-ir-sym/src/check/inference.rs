use std::sync::Arc;

use dada_ir_ast::span::Span;

use crate::{
    check::universe::Universe,
    ir::{indices::InferVarIndex, types::SymGenericKind},
};

use super::{
    chains::{Chain, RedTy},
    predicates::Predicate,
    report::{ArcOrElse, OrElse},
};

pub(crate) struct InferenceVarData<'db> {
    #[expect(dead_code)]
    universe: Universe,

    span: Span<'db>,

    /// If the element for a given predicate is `Some`, then the predicate is known to be true
    /// due to code at the given span. If the element is `None`, then it is not known that the
    /// predicate is true (but it still could be, depending on what value we ultimately infer).
    ///
    /// See also the `isnt` field.
    is: [Option<ArcOrElse<'db>>; Predicate::LEN],

    /// If the element for a given predicate is `Some`, then the predicate is NOT known to be true
    /// due to code at the given span.
    ///
    /// This is a subtle distinction. Knowing that a variable `isnt (known to be) copy` doesn't
    /// imply that it is `is (known to be) move`. It means "you will never be able to prove this is copy".
    isnt: [Option<ArcOrElse<'db>>; Predicate::LEN],

    /// Bounds on this variable suitable for its kind.
    bounds: InferenceVarBounds<'db>,
}

impl<'db> InferenceVarData<'db> {
    fn new(universe: Universe, span: Span<'db>, bounds: InferenceVarBounds<'db>) -> Self {
        Self {
            universe,
            span,
            bounds,
            is: [None, None, None, None],
            isnt: [None, None, None, None],
        }
    }

    /// Create the data for a new permission inference variable.
    pub fn new_perm(universe: Universe, span: Span<'db>) -> Self {
        Self::new(universe, span, InferenceVarBounds::Perm {
            lower: Default::default(),
            upper: Default::default(),
        })
    }

    /// Create the data for a new type inference variable.
    /// Requires the index `perm` of a corresponding permission variable.
    pub fn new_ty(universe: Universe, span: Span<'db>, perm: InferVarIndex) -> Self {
        Self::new(universe, span, InferenceVarBounds::Ty {
            perm,
            lower: Default::default(),
            upper: Default::default(),
        })
    }

    /// Returns the span of code which triggered the inference variable to be created.
    pub fn span(&self) -> Span<'db> {
        self.span
    }

    /// Returns the kind of the inference variable.
    pub fn kind(&self) -> SymGenericKind {
        match self.bounds {
            InferenceVarBounds::Perm { .. } => SymGenericKind::Perm,
            InferenceVarBounds::Ty { .. } => SymGenericKind::Type,
        }
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

    /// Access to the bounds on the inference variable.
    pub fn bounds(&self) -> &InferenceVarBounds<'db> {
        &self.bounds
    }

    /// Mutable access to the bounds on the inference variable.
    pub fn bounds_mut(&mut self) -> &mut InferenceVarBounds<'db> {
        &mut self.bounds
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
    ) -> Option<ArcOrElse<'db>> {
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
    ) -> Option<ArcOrElse<'db>> {
        assert!(self.is_known_to_provably_be(predicate).is_none());
        if self.is_known_not_to_provably_be(predicate).is_none() {
            let or_else = or_else.to_arc();
            self.isnt[predicate.index()] = Some(or_else.clone());
            Some(or_else)
        } else {
            None
        }
    }

    /// Returns the lower bounds on this permission variable.
    ///
    /// # Panics
    ///
    /// If this is not a permission variable.
    pub fn lower_chains(&self) -> &[(Chain<'db>, ArcOrElse<'db>)] {
        match &self.bounds {
            InferenceVarBounds::Perm { lower, .. } => lower,
            _ => panic!("lower_chains invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Returns the upper bounds on this permission variable.
    ///
    /// # Panics
    ///
    /// If this is not a permission variable.
    pub fn upper_chains(&self) -> &[(Chain<'db>, ArcOrElse<'db>)] {
        match &self.bounds {
            InferenceVarBounds::Perm { upper, .. } => upper,
            _ => panic!("lower_chains invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Returns the permission variable corresponding to this type variable.
    ///
    /// # Panics
    ///
    /// If this is not a type variable.
    pub fn perm(&self) -> InferVarIndex {
        match &self.bounds {
            InferenceVarBounds::Ty { perm, .. } => *perm,
            _ => panic!("perm invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Returns the lower bound on this type variable.
    ///
    /// # Panics
    ///
    /// If this is not a type variable.
    pub fn lower_red_ty(&self) -> Option<(RedTy<'db>, ArcOrElse<'db>)> {
        match &self.bounds {
            InferenceVarBounds::Ty { lower, .. } => lower.clone(),
            _ => panic!("lower_red_ty invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Returns the upper bound on this type variable.
    ///
    /// # Panics
    ///
    /// If this is not a type variable.
    pub fn upper_red_ty(&self) -> Option<(RedTy<'db>, ArcOrElse<'db>)> {
        match &self.bounds {
            InferenceVarBounds::Ty { upper, .. } => upper.clone(),
            _ => panic!("upper_red_ty invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Insert a chain as a lower bound.
    /// Returns `Some(or_else.to_arc())` if this is a new upper bound.
    pub fn insert_lower_chain(
        &mut self,
        chain: &Chain<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> Option<ArcOrElse<'db>> {
        let lower_chains = match &mut self.bounds {
            InferenceVarBounds::Perm { lower, .. } => lower,
            _ => panic!(
                "insert_lower_chain invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        if lower_chains.iter().any(|pair| pair.0 == *chain) {
            return None;
        }
        let or_else = or_else.to_arc();
        lower_chains.push((chain.clone(), or_else.clone()));
        Some(or_else)
    }

    /// Insert a chain as an upper bound.
    /// Returns `Some(or_else.to_arc())` if this is a new upper bound.
    pub fn insert_upper_chain(
        &mut self,
        chain: &Chain<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> Option<ArcOrElse<'db>> {
        let upper_chains = match &mut self.bounds {
            InferenceVarBounds::Perm { upper, .. } => upper,
            _ => panic!(
                "insert_upper_chain invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        if upper_chains.iter().any(|pair| pair.0 == *chain) {
            return None;
        }
        let or_else = or_else.to_arc();
        upper_chains.push((chain.clone(), or_else.clone()));
        Some(or_else)
    }

    /// Set the lower bounding red ty. Returns `c` with the `or_else` reason if
    /// this is a new value for the upper bounding red ty.
    ///
    /// # Panics
    ///
    /// If there is already a lower bound red ty.
    pub fn set_lower_red_ty(
        &mut self,
        red_ty: RedTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> ArcOrElse<'db> {
        let lower_red_ty = match &mut self.bounds {
            InferenceVarBounds::Ty { lower, .. } => lower,
            _ => panic!(
                "set_lower_red_ty invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        assert!(lower_red_ty.is_none());
        let or_else = or_else.to_arc();
        *lower_red_ty = Some((red_ty, or_else.clone()));
        or_else
    }

    /// Set the upper bounding red ty. Returns `Some(c)` with the `or_else` reason if
    /// this is a new value for the upper bounding red ty.
    ///
    /// # Panics
    ///
    /// If there is already an upper bound red ty.
    pub fn set_upper_red_ty(
        &mut self,
        red_ty: RedTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> ArcOrElse<'db> {
        let upper_red_ty = match &mut self.bounds {
            InferenceVarBounds::Ty { upper, .. } => upper,
            _ => panic!(
                "set_upper_red_ty invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        assert!(upper_red_ty.is_none());
        let or_else = or_else.to_arc();
        *upper_red_ty = Some((red_ty, or_else.clone()));
        or_else
    }
}

enum InferenceVarBounds<'db> {
    Perm {
        lower: Vec<(Chain<'db>, ArcOrElse<'db>)>,
        upper: Vec<(Chain<'db>, ArcOrElse<'db>)>,
    },

    Ty {
        perm: InferVarIndex,
        lower: Option<(RedTy<'db>, ArcOrElse<'db>)>,
        upper: Option<(RedTy<'db>, ArcOrElse<'db>)>,
    },
}

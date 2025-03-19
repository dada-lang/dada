use std::sync::Arc;

use dada_ir_ast::span::Span;
use salsa::Update;

use crate::ir::{indices::InferVarIndex, types::SymGenericKind};

use super::{
    predicates::Predicate,
    red::{Chain, RedTy},
    report::{ArcOrElse, OrElse},
};

mod serialize;

pub(crate) struct InferenceVarData<'db> {
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
    fn new(span: Span<'db>, bounds: InferenceVarBounds<'db>) -> Self {
        Self {
            span,
            bounds,
            is: [None, None, None, None],
            isnt: [None, None, None, None],
        }
    }

    /// Create the data for a new permission inference variable.
    pub fn new_perm(span: Span<'db>) -> Self {
        Self::new(
            span,
            InferenceVarBounds::Perm {
                lower: Default::default(),
                upper: Default::default(),
            },
        )
    }

    /// Create the data for a new type inference variable.
    /// Requires the index `perm` of a corresponding permission variable.
    pub fn new_ty(span: Span<'db>, perm: InferVarIndex) -> Self {
        Self::new(
            span,
            InferenceVarBounds::Ty {
                perm,
                lower: Default::default(),
                upper: Default::default(),
            },
        )
    }

    /// Returns the span of code which triggered the inference variable to be created.
    pub fn span(&self) -> Span<'db> {
        self.span
    }

    /// Returns the kind of the inference variable.
    pub fn kind(&self) -> InferVarKind {
        match self.bounds {
            InferenceVarBounds::Perm { .. } => InferVarKind::Perm,
            InferenceVarBounds::Ty { .. } => InferVarKind::Type,
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

        // If these constraints are already recorded, just return.
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
    #[track_caller]
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
    #[track_caller]
    pub fn upper_chains(&self) -> &[(Chain<'db>, ArcOrElse<'db>)] {
        match &self.bounds {
            InferenceVarBounds::Perm { upper, .. } => upper,
            _ => panic!("lower_chains invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Returns the permission variable corresponding to this type variable.
    /// Returns `None` if this is a permission variable.
    #[track_caller]
    pub fn perm(&self) -> Option<InferVarIndex> {
        match &self.bounds {
            InferenceVarBounds::Ty { perm, .. } => Some(*perm),
            InferenceVarBounds::Perm { .. } => None,
        }
    }

    /// Returns the lower bound on this type variable.
    ///
    /// # Panics
    ///
    /// If this is not a type variable.
    #[track_caller]
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
    #[track_caller]
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum InferVarKind {
    Type,
    Perm,
}

impl From<InferVarKind> for SymGenericKind {
    fn from(value: InferVarKind) -> Self {
        match value {
            InferVarKind::Type => SymGenericKind::Type,
            InferVarKind::Perm => SymGenericKind::Perm,
        }
    }
}

pub enum InferenceVarBounds<'db> {
    /// Bounds for a permission:
    ///
    /// The inferred permission `?P` must meet
    ///
    /// * `L <: ?P` for each `L` in `lower`
    /// * `U <: ?P` for each `U` in `upper`
    ///
    /// This in turn implies that `L <: U`
    /// for all `L in lower`, `U in upper`.
    Perm {
        lower: Vec<(Chain<'db>, ArcOrElse<'db>)>,
        upper: Vec<(Chain<'db>, ArcOrElse<'db>)>,
    },

    /// Bounds for a type:
    ///
    /// The inferred type `?T` must
    ///
    /// * have a red-perm of `perm`
    ///   (we always create an associated permission
    ///   variable for every type variable)
    /// * have a red-ty `R` where `lower <= R`
    /// * have a red-ty `R` where `R <= upper`
    Ty {
        perm: InferVarIndex,
        lower: Option<(RedTy<'db>, ArcOrElse<'db>)>,
        upper: Option<(RedTy<'db>, ArcOrElse<'db>)>,
    },
}

/// Trait implemented by types returned by mutation methods
/// like [`InferenceVarData::insert_lower_infer_bound`][]
/// or [`InferenceVarData::set_lower_red_ty`][].
/// Can be used to check if those return values indicate that
/// the inference var data was actually changed.
pub trait InferenceVarDataChanged {
    fn did_change(&self) -> bool;
}

impl InferenceVarDataChanged for bool {
    fn did_change(&self) -> bool {
        *self
    }
}

impl InferenceVarDataChanged for Option<ArcOrElse<'_>> {
    fn did_change(&self) -> bool {
        self.is_some()
    }
}

impl InferenceVarDataChanged for ArcOrElse<'_> {
    fn did_change(&self) -> bool {
        true
    }
}

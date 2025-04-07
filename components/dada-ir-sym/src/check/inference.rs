use dada_ir_ast::span::Span;
use salsa::Update;

use crate::ir::{indices::InferVarIndex, types::SymGenericKind};

use super::{
    predicates::Predicate,
    red::{Chain, RedTy},
    report::{ArcOrElse, OrElse},
};

mod serialize;

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    FromBelow,
    FromAbove,
}

pub(crate) struct InferenceVarData<'db> {
    span: Span<'db>,

    /// If the element for a given predicate is `Some`, then the predicate is known to be true
    /// for this inference variable due to code at the given span. If the element is `None`,
    /// then it is not known that the predicate is true (but it still could be, depending
    /// on what value we ultimately infer).
    ///
    /// **Subtle:** For ty inference variables, this applies only to the red-ty bounds.
    /// The associated permission variable tracks predicates separately.
    ///
    /// See also the `isnt` field.
    is: [Option<ArcOrElse<'db>>; Predicate::LEN],

    /// Like [`Self::is`][] except it records if the predicate is known to not be provable.
    /// Note that knowing that a variable `isnt (known to be) copy` doesn't
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

    /// Returns `Some(s)` if the predicate is known to be in the [`is`](`Self::is`) set.
    pub fn is_known_to_provably_be(&self, predicate: Predicate) -> Option<ArcOrElse<'db>> {
        self.is[predicate.index()].clone()
    }

    /// Returns `Some(s)` if the predicate is found in the [`isnt`](`Self::isnt`) set.
    pub fn is_known_not_to_provably_be(&self, predicate: Predicate) -> Option<ArcOrElse<'db>> {
        self.isnt[predicate.index()].clone()
    }

    /// Insert a predicate into the [`is`](`Self::is`) set and its invert into the [`isnt`](`Self::isnt`) set.
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

    /// Insert a predicate into the [`isnt`](`Self::isnt`) set,
    /// returning `Some` if this is a new bound.
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

    /// Returns the upper or lower bounds on this permission variable.
    ///
    /// # Panics
    ///
    /// If this is not a permission variable.
    #[track_caller]
    pub fn chain_bounds(&self, direction: Direction) -> &[(Chain<'db>, ArcOrElse<'db>)] {
        match &self.bounds {
            InferenceVarBounds::Perm { lower, upper, .. } => match direction {
                Direction::FromBelow => lower,
                Direction::FromAbove => upper,
            },
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

    /// Returns the lower or upper bound on this type variable, depending on `direction`.
    ///
    /// # Panics
    ///
    /// If this is not a type variable.
    #[track_caller]
    pub fn red_ty_bound(&self, direction: Direction) -> Option<(RedTy<'db>, ArcOrElse<'db>)> {
        match &self.bounds {
            InferenceVarBounds::Ty { lower, upper, .. } => match direction {
                Direction::FromBelow => lower.clone(),
                Direction::FromAbove => upper.clone(),
            },
            _ => panic!("red_ty_bound invoked on a var of kind `{:?}`", self.kind()),
        }
    }

    /// Insert a chain as a lower bound.
    /// Returns `Some(or_else.to_arc())` if this is a new upper bound.
    pub fn insert_chain_bound(
        &mut self,
        chain: &Chain<'db>,
        direction: Direction,
        or_else: &dyn OrElse<'db>,
    ) -> Option<ArcOrElse<'db>> {
        let chain_bounds = match &mut self.bounds {
            InferenceVarBounds::Perm { lower, upper, .. } => match direction {
                Direction::FromBelow => lower,
                Direction::FromAbove => upper,
            },
            _ => panic!(
                "insert_lower_chain invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        if chain_bounds.iter().any(|pair| pair.0 == *chain) {
            return None;
        }
        let or_else = or_else.to_arc();
        chain_bounds.push((chain.clone(), or_else.clone()));
        Some(or_else)
    }

    /// Overwrite the lower or upper bounding red ty, depending on `direction`.
    /// Returns the [to_arc'd](`OrElse::to_arc`) version of `or_else`.
    pub fn set_red_ty_bound(
        &mut self,
        direction: Direction,
        red_ty: RedTy<'db>,
        or_else: &dyn OrElse<'db>,
    ) {
        let red_ty_bound = match &mut self.bounds {
            InferenceVarBounds::Ty { lower, upper, .. } => match direction {
                Direction::FromBelow => lower,
                Direction::FromAbove => upper,
            },
            _ => panic!(
                "set_lower_red_ty invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        *red_ty_bound = Some((red_ty, or_else.to_arc()));
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
    /// * `?P <: U` for each `U` in `upper`
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

impl InferenceVarDataChanged for () {
    fn did_change(&self) -> bool {
        true
    }
}

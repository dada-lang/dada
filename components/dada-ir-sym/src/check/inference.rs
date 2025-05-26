//! Type and permission inference for Dada.
#![doc = include_str!("../../docs/type_inference.md")]

use dada_ir_ast::span::Span;
use salsa::Update;
use serde::Serialize;

use crate::ir::{indices::InferVarIndex, types::SymGenericKind};

use super::{
    red::{RedPerm, RedTy},
    report::{ArcOrElse, OrElse},
};

mod serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Direction {
    FromBelow,
    FromAbove,
}

pub(crate) struct InferenceVarData<'db> {
    span: Span<'db>,

    /// Bounds on this variable suitable for its kind.
    bounds: InferenceVarBounds<'db>,
}

impl<'db> InferenceVarData<'db> {
    fn new(span: Span<'db>, bounds: InferenceVarBounds<'db>) -> Self {
        Self { span, bounds }
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

    /// Returns the upper or lower bounds on this permission variable.
    ///
    /// # Panics
    ///
    /// If this is not a permission variable.
    #[track_caller]
    pub fn red_perm_bound(&self, direction: Direction) -> Option<(RedPerm<'db>, ArcOrElse<'db>)> {
        match &self.bounds {
            InferenceVarBounds::Perm { lower, upper, .. } => match direction {
                Direction::FromBelow => lower.clone(),
                Direction::FromAbove => upper.clone(),
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

    /// Insert a red perm as a (lower|upper) bound.
    /// Returns `Some(or_else.to_arc())` if this is a new (lower|upper) bound.
    pub fn set_red_perm_bound(
        &mut self,
        direction: Direction,
        red_perm: RedPerm<'db>,
        or_else: &dyn OrElse<'db>,
    ) {
        let perm_bound = match &mut self.bounds {
            InferenceVarBounds::Perm { lower, upper, .. } => match direction {
                Direction::FromBelow => lower,
                Direction::FromAbove => upper,
            },
            _ => panic!(
                "insert_lower_chain invoked on a var of kind `{:?}`",
                self.kind()
            ),
        };
        *perm_bound = Some((red_perm, or_else.to_arc()));
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
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
        lower: Option<(RedPerm<'db>, ArcOrElse<'db>)>,
        upper: Option<(RedPerm<'db>, ArcOrElse<'db>)>,
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
/// like `InferenceVarData::set_red_perm_bound`
/// or `InferenceVarData::set_red_ty_bound`.
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

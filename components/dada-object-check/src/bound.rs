use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use dada_ir_sym::{
    indices::SymInferVarIndex,
    symbol::{HasKind, SymGenericKind},
    ty::{SymGenericTerm, SymPerm, SymTy},
};

use crate::{
    check::Runtime,
    inference::InferenceVarData,
    object_ir::{ObjectGenericTerm, ObjectTy, ObjectTyKind},
};

/// Either a lower or upper bound on an inference variable `?X`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Bound<Term> {
    /// A bound `B` where `B <: ?X` -- intuitively, `B`
    /// is a value that flows *into* the inference variable.
    LowerBound(Term),

    /// A bound `B` where `?X <: B` -- intuitively, `B`
    /// is a value that is *read out* from the inference variable.
    UpperBound(Term),
}

pub(crate) trait BoundedTerm<'db>: HasKind<'db> {
    type Type;
    fn assert_type(self, db: &'db dyn crate::Db) -> Self::Type;
}

impl<'db, Term: BoundedTerm<'db>> HasKind<'db> for Bound<Term> {
    fn has_kind(&self, db: &'db dyn crate::Db, kind: SymGenericKind) -> bool {
        match self {
            Bound::LowerBound(ty) => ty.has_kind(db, kind),
            Bound::UpperBound(ty) => ty.has_kind(db, kind),
        }
    }
}

impl<'db, Term: BoundedTerm<'db>> Bound<Term> {
    pub fn assert_type(self, db: &'db dyn crate::Db) -> Bound<Term::Type> {
        match self {
            Bound::LowerBound(term) => Bound::LowerBound(term.assert_type(db)),
            Bound::UpperBound(term) => Bound::UpperBound(term.assert_type(db)),
        }
    }

    pub fn into_term(self) -> Term {
        match self {
            Bound::LowerBound(term) => term,
            Bound::UpperBound(term) => term,
        }
    }
}

impl<'db> BoundedTerm<'db> for SymGenericTerm<'db> {
    type Type = SymTy<'db>;

    fn assert_type(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        self.assert_type(db)
    }
}

impl<'db> BoundedTerm<'db> for SymTy<'db> {
    type Type = SymTy<'db>;

    fn assert_type(self, _db: &'db dyn crate::Db) -> SymTy<'db> {
        self
    }
}

impl<'db> BoundedTerm<'db> for ObjectGenericTerm<'db> {
    type Type = ObjectTy<'db>;

    fn assert_type(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self.assert_type(db)
    }
}

impl<'db> BoundedTerm<'db> for ObjectTy<'db> {
    type Type = ObjectTy<'db>;

    fn assert_type(self, _db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }
}

impl<'db> From<Bound<SymTy<'db>>> for Bound<SymGenericTerm<'db>> {
    fn from(value: Bound<SymTy<'db>>) -> Self {
        match value {
            Bound::LowerBound(v) => Bound::LowerBound(v.into()),
            Bound::UpperBound(v) => Bound::UpperBound(v.into()),
        }
    }
}

impl<'db> BoundedTerm<'db> for SymPerm<'db> {
    type Type = SymTy<'db>;

    fn assert_type(self, _db: &'db dyn crate::Db) -> SymTy<'db> {
        panic!("expected a type, found a perm: `{self:?}`")
    }
}

impl<'db> From<Bound<SymPerm<'db>>> for Bound<SymGenericTerm<'db>> {
    fn from(value: Bound<SymPerm<'db>>) -> Self {
        match value {
            Bound::LowerBound(v) => Bound::LowerBound(v.into()),
            Bound::UpperBound(v) => Bound::UpperBound(v.into()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Direction {
    LowerBounds,
    UpperBounds,
}

impl Direction {
    fn bounds<'i, 'db, Term>(self, data: &'i InferenceVarData<'db>) -> &'i [Term]
    where
        Term: OutputTerm<'db>,
    {
        let (lower, upper) = Term::bound_slices(data);
        match self {
            Direction::LowerBounds => lower,
            Direction::UpperBounds => upper,
        }
    }
}

pub(crate) struct TransitiveBounds<'db, Term: OutputTerm<'db>> {
    runtime: Runtime<'db>,
    direction: Direction,
    inference_vars: Vec<(SymInferVarIndex, usize)>,
    phantom: PhantomData<fn() -> Term>,
}

impl<'db, Term: OutputTerm<'db>> TransitiveBounds<'db, Term> {
    pub fn new(check: &Runtime<'db>, direction: Direction, var: SymInferVarIndex) -> Self {
        Self {
            runtime: check.clone(),
            direction,
            inference_vars: std::iter::once((var, 0)).collect(),
            phantom: PhantomData,
        }
    }

    fn push_inference_var(&mut self, var: SymInferVarIndex) {
        if self.inference_vars.iter().any(|(infer, _)| *infer == var) {
            return;
        }
        self.inference_vars.push((var, 0));
    }

    fn poll_next_impl(&mut self, cx: &mut Context<'_>) -> Poll<Option<Term>> {
        // Iterate through all of the inference variables whose bounds we are waiting for.
        // Note that we may discover new variables as we iterate, so we don't use an iterator,
        // but rather track the current index (new variables will be pushed at the end).
        let mut inference_vars_index = 0;
        while inference_vars_index < self.inference_vars.len() {
            // Load the variable and the current index into its (upper|lower) bounds.
            let (infer, bound_index) = &mut self.inference_vars[inference_vars_index];
            inference_vars_index += 1;

            // Check if there is a bound with `bound_index` for this variable
            let next_bound = self.runtime.with_inference_var_data(*infer, |data| {
                let bounds: &[Term] = self.direction.bounds(data);
                bounds.get(*bound_index).copied()
            });

            // If there is a bound, then check whether the bound is an inference variable.
            // If not, we can return it.
            // If so, we push it onto the list of variables to wait on, and continue.
            if let Some(bound) = next_bound {
                *bound_index += 1;

                let Some(var) = bound.as_var(self.runtime.db) else {
                    return Poll::Ready(Some(bound));
                };

                self.push_inference_var(var);
            }
        }

        // If there are no more bounds being produced, we've reached the end of the stream.
        if self.runtime.check_complete() {
            return Poll::Ready(None);
        }

        // Otherwise block on all the variables we've seen so far.
        // A new bound for any of them will wake us up.
        for (var, current_index) in &mut self.inference_vars {
            self.runtime.block_on_inference_var(*var, cx);
        }

        Poll::Pending
    }
}

impl<'db, Term: OutputTerm<'db>> futures::Stream for TransitiveBounds<'db, Term> {
    type Item = Term;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_impl(cx)
    }
}

pub(crate) trait OutputTerm<'db>: Copy {
    fn bound_slices<'i>(data: &'i InferenceVarData<'db>) -> (&'i [Self], &'i [Self]);

    fn as_var(self, db: &'db dyn crate::Db) -> Option<SymInferVarIndex>;
}

impl<'db> OutputTerm<'db> for ObjectGenericTerm<'db> {
    fn bound_slices<'i>(data: &'i InferenceVarData<'db>) -> (&'i [Self], &'i [Self]) {
        (data.lower_bounds(), data.upper_bounds())
    }

    fn as_var(self, db: &'db dyn crate::Db) -> Option<SymInferVarIndex> {
        match self {
            ObjectGenericTerm::Type(object_ty) => match object_ty.kind(db) {
                ObjectTyKind::Infer(infer) => Some(*infer),
                ObjectTyKind::Var(..)
                | ObjectTyKind::Named(..)
                | ObjectTyKind::Never
                | ObjectTyKind::Error(_) => None,
            },
            ObjectGenericTerm::Perm => None,
            ObjectGenericTerm::Place => None,
            ObjectGenericTerm::Error(_) => None,
        }
    }
}

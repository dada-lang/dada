use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    ir::indices::InferVarIndex,
    ir::types::{SymGenericTerm, SymPermKind, SymPlaceKind, SymTyKind},
};

use crate::{check::Runtime, inference::InferenceVarData};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Direction {
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

    pub fn infer_var_bounds<'i, 'db, Term>(self, data: &'i InferenceVarData<'db>) -> &'i [Term]
    where
        Term: OutputTerm<'db>,
    {
        let (lower, upper) = Term::bound_slices(data);
        match self {
            Direction::LowerBoundedBy => lower,
            Direction::UpperBoundedBy => upper,
        }
    }
}

pub(crate) struct TransitiveBounds<'db, Term: OutputTerm<'db>> {
    runtime: Runtime<'db>,
    direction: Direction,
    inference_vars: Vec<(InferVarIndex, usize)>,
    phantom: PhantomData<fn() -> Term>,
}

impl<'db, Term: OutputTerm<'db>> TransitiveBounds<'db, Term> {
    pub fn new(check: &Runtime<'db>, direction: Direction, var: InferVarIndex) -> Self {
        Self {
            runtime: check.clone(),
            direction,
            inference_vars: std::iter::once((var, 0)).collect(),
            phantom: PhantomData,
        }
    }

    fn push_inference_var(&mut self, var: InferVarIndex) {
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
                let bounds: &[Term] = self.direction.infer_var_bounds(data);
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
        for &mut (var, _) in &mut self.inference_vars {
            self.runtime.block_on_inference_var(var, cx);
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

    fn as_var(self, db: &'db dyn crate::Db) -> Option<InferVarIndex>;
}

impl<'db> OutputTerm<'db> for SymGenericTerm<'db> {
    fn bound_slices<'i>(data: &'i InferenceVarData<'db>) -> (&'i [Self], &'i [Self]) {
        (data.lower_bounds(), data.upper_bounds())
    }

    fn as_var(self, db: &'db dyn crate::Db) -> Option<InferVarIndex> {
        match self {
            SymGenericTerm::Type(ty) => match ty.kind(db) {
                SymTyKind::Infer(infer) => Some(*infer),
                SymTyKind::Var(..)
                | SymTyKind::Named(..)
                | SymTyKind::Never
                | SymTyKind::Error(_)
                | SymTyKind::Perm(..) => None,
            },
            SymGenericTerm::Perm(perm) => match perm.kind(db) {
                SymPermKind::Infer(infer) => Some(*infer),
                SymPermKind::My
                | SymPermKind::Our
                | SymPermKind::Shared(_)
                | SymPermKind::Leased(_)
                | SymPermKind::Given(_)
                | SymPermKind::Var(_)
                | SymPermKind::Error(_) => None,
            },
            SymGenericTerm::Place(place) => match place.kind(db) {
                SymPlaceKind::Infer(infer) => Some(*infer),
                SymPlaceKind::Var(_)
                | SymPlaceKind::Field(..)
                | SymPlaceKind::Index(..)
                | SymPlaceKind::Error(..) => None,
            },
            SymGenericTerm::Error(_) => None,
        }
    }
}

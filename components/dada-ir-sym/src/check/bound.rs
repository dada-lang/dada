use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use dada_util::{debug, debug_heading, log::DebugArgument};

use crate::ir::{
    indices::InferVarIndex,
    inference::Direction,
    types::{SymGenericKind, SymGenericTerm, SymTy},
};

use crate::check::runtime::Runtime;

/// A stream of the terms that bound an inference variable `?X`.
///
/// If `?X` is bounded by another variable `?Y`, then the bounds of `?Y`
/// are returned (but not `?Y`) itself.
///
/// Using [`Self::push_inference_var`], you can manually add new inference variables
/// to the stream whose bounds will be returned as well.
pub(crate) struct TransitiveBounds<'db, Kind: OutputKind<'db>> {
    runtime: Runtime<'db>,
    direction: Direction,
    inference_vars: Vec<(InferVarIndex, usize)>,
    first_bound: Option<Kind>,
    phantom: PhantomData<fn() -> Kind>,
}

impl<'db, Kind: OutputKind<'db>> TransitiveBounds<'db, Kind> {
    /// Create a new stream of bounds for an inference variable.
    pub fn new(check: &Runtime<'db>, direction: Direction, var: InferVarIndex) -> Self {
        let mut this = Self {
            runtime: check.clone(),
            direction,
            inference_vars: vec![],
            first_bound: None,
            phantom: PhantomData,
        };

        this.push_inference_var(var);

        this
    }

    /// Create a stream that (initially) yields just a single term.
    /// Users may invoke [`Self::push_inference_var`] in response,
    /// which will cause it to yield more terms.
    pub fn just(check: &Runtime<'db>, direction: Direction, term: Kind) -> Self {
        Self {
            runtime: check.clone(),
            direction,
            inference_vars: vec![],
            first_bound: Some(term),
            phantom: PhantomData,
        }
    }

    /// Push a new inference variable onto the list of variables to wait on.
    pub(crate) fn push_inference_var(&mut self, var: InferVarIndex) {
        if self.inference_vars.iter().any(|(infer, _)| *infer == var) {
            return;
        }

        // check the variable is of the correct kind
        let kind = self
            .runtime
            .with_inference_var_data(var, |data| data.kind());
        assert_eq!(Kind::KIND, kind);

        self.inference_vars.push((var, 0));
    }

    fn poll_next_impl(&mut self, cx: &mut Context<'_>) -> Poll<Option<Kind>> {
        debug_heading!("poll_next_impl");
        let db = self.runtime.db;

        if let Some(first_bound) = self.first_bound.take() {
            debug!("yielding first bound", first_bound);
            return Poll::Ready(Some(first_bound));
        }

        // Iterate through all of the inference variables whose bounds we are waiting for.
        // Note that we may discover new variables as we iterate, so we don't use an iterator,
        // but rather track the current index (new variables will be pushed at the end).
        let mut inference_vars_index = 0;
        while inference_vars_index < self.inference_vars.len() {
            // Load the variable and the current index into its (upper|lower) bounds.
            let (infer, bound_index) = &mut self.inference_vars[inference_vars_index];
            inference_vars_index += 1;

            debug!("inference var", inference_vars_index, infer, bound_index);

            // Check if there is a bound with `bound_index` for this variable
            let next_bound = self.runtime.with_inference_var_data(*infer, |data| {
                let bounds = self.direction.infer_var_bounds(data);
                bounds.get(*bound_index).copied()
            });

            debug!("next bound", next_bound);
            // If there is a bound, then check whether the bound is an inference variable.
            // If not, we can return it.
            // If so, we push it onto the list of variables to wait on, and continue.
            if let Some(bound) = next_bound {
                *bound_index += 1;

                let Some(var) = bound.as_infer(db) else {
                    debug!("yielding bound");
                    return Poll::Ready(Some(Kind::assert_from_term(db, bound)));
                };

                debug!("pushing inference var", var);
                self.push_inference_var(var);
            }
        }

        // If there are no more bounds being produced, we've reached the end of the stream.
        if self.runtime.check_complete() {
            debug!("check complete");
            return Poll::Ready(None);
        }

        // Otherwise block on all the variables we've seen so far.
        // A new bound for any of them will wake us up.
        for &(var, _) in &self.inference_vars {
            debug!("blocking on inference var", var);
            self.runtime.block_on_inference_var(var, cx);
        }

        debug!("pending");
        Poll::Pending
    }
}

impl<'db, Term: OutputKind<'db>> futures::Stream for TransitiveBounds<'db, Term> {
    type Item = Term;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_impl(cx)
    }
}

/// Trait implemented by types representing some specific kind of [`SymGenericTerm`],
/// e.g. [`SymTy`].
pub(crate) trait OutputKind<'db>: Copy + DebugArgument + Unpin {
    /// The kind of term that this type represents.
    const KIND: SymGenericKind;

    /// Assert that a term is of the correct kind, and convert it to the type.
    fn assert_from_term(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self;
}

impl<'db> OutputKind<'db> for SymTy<'db> {
    const KIND: SymGenericKind = SymGenericKind::Type;

    fn assert_from_term(db: &'db dyn crate::Db, term: SymGenericTerm<'db>) -> Self {
        term.assert_type(db)
    }
}

use std::{
    marker::PhantomData,
    pin::Pin,
    task::{ready, Context, Poll},
};

use dada_ir_sym::{
    indices::SymInferVarIndex,
    symbol::{HasKind, SymGenericKind},
    ty::{SymGenericTerm, SymPerm, SymTy},
};

use crate::{
    check::Runtime,
    inference::InferenceVarData,
    object_ir::{ObjectGenericTerm, ObjectTy},
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

/// A stream over the bounds on an inference variable.
pub(crate) struct InferenceVarBounds<'db, Term: OutputTerm<'db>> {
    runtime: Runtime<'db>,
    inference_var: SymInferVarIndex,
    upper_bounds: usize,
    lower_bounds: usize,
    phantom: PhantomData<fn() -> Term>,
}

impl<'db, Term: OutputTerm<'db>> InferenceVarBounds<'db, Term> {
    pub fn new(check: &Runtime<'db>, inference_var: SymInferVarIndex) -> Self {
        Self {
            runtime: check.clone(),
            inference_var,
            upper_bounds: 0,
            lower_bounds: 0,
            phantom: PhantomData,
        }
    }
}

impl<'db, Term: OutputTerm<'db>> futures::Stream for InferenceVarBounds<'db, Term> {
    type Item = Bound<Term>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let check = self.runtime.clone();
        let next_bound = check.with_inference_var_data(self.inference_var, |data| {
            let &Self {
                lower_bounds,
                upper_bounds,
                ..
            } = &*self;

            let (data_lower_bounds, data_upper_bounds) = Term::bound_slices(data);

            if lower_bounds < data_lower_bounds.len() {
                self.lower_bounds += 1;
                Some(Bound::LowerBound(data_lower_bounds[lower_bounds]))
            } else if upper_bounds < data_upper_bounds.len() {
                self.upper_bounds += 1;
                Some(Bound::UpperBound(data_upper_bounds[upper_bounds]))
            } else {
                None
            }
        });

        match next_bound {
            Some(bound) => Poll::Ready(Some(bound)),
            None => {
                let () = ready!(self.runtime.block_on_inference_var(self.inference_var, cx));
                Poll::Ready(None)
            }
        }
    }
}

pub(crate) trait OutputTerm<'db>: Copy {
    fn bound_slices<'i>(data: &'i InferenceVarData<'db>) -> (&'i [Self], &'i [Self]);
}

impl<'db> OutputTerm<'db> for ObjectGenericTerm<'db> {
    fn bound_slices<'i>(data: &'i InferenceVarData<'db>) -> (&'i [Self], &'i [Self]) {
        (data.lower_bounds(), data.upper_bounds())
    }
}

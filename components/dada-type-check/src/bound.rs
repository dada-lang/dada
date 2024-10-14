use std::{
    pin::Pin,
    task::{ready, Context, Poll},
};

use dada_ir_sym::{
    indices::SymVarIndex,
    symbol::SymGenericKind,
    ty::{SymGenericTerm, SymPerm, SymTy},
};

use crate::executor::Check;

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

pub(crate) trait BoundTerm<'db> {
    fn has_kind(self, kind: SymGenericKind) -> bool;
    fn assert_type(self, db: &'db dyn crate::Db) -> SymTy<'db>;
}

impl<'db, Term: BoundTerm<'db>> Bound<Term> {
    pub fn has_kind(self, kind: SymGenericKind) -> bool {
        match self {
            Bound::LowerBound(ty) => ty.has_kind(kind),
            Bound::UpperBound(ty) => ty.has_kind(kind),
        }
    }

    pub fn assert_type(self, db: &'db dyn crate::Db) -> Bound<SymTy<'db>> {
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

impl<'db> BoundTerm<'db> for SymGenericTerm<'db> {
    fn has_kind(self, kind: SymGenericKind) -> bool {
        self.has_kind(kind)
    }

    fn assert_type(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        self.assert_type(db)
    }
}

impl<'db> BoundTerm<'db> for SymTy<'db> {
    fn has_kind(self, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Type
    }

    fn assert_type(self, _db: &'db dyn crate::Db) -> SymTy<'db> {
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

impl<'db> BoundTerm<'db> for SymPerm<'db> {
    fn has_kind(self, kind: SymGenericKind) -> bool {
        kind == SymGenericKind::Type
    }

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
pub(crate) struct InferenceVarBounds<'chk, 'db> {
    check: Check<'chk, 'db>,
    inference_var: SymVarIndex,
    upper_bounds: usize,
    lower_bounds: usize,
}

impl<'chk, 'db> InferenceVarBounds<'chk, 'db> {
    pub fn new(check: &Check<'chk, 'db>, inference_var: SymVarIndex) -> Self {
        Self {
            check: check.clone(),
            inference_var,
            upper_bounds: 0,
            lower_bounds: 0,
        }
    }
}

impl<'chk, 'db> futures::Stream for InferenceVarBounds<'chk, 'db> {
    type Item = Bound<SymGenericTerm<'db>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let check = self.check.clone();
        let next_bound = check.with_inference_var_data(self.inference_var, |data| {
            let &Self {
                upper_bounds,
                lower_bounds,
                ..
            } = &*self;
            if upper_bounds < data.upper_bounds().len() {
                self.upper_bounds += 1;
                Some(Bound::UpperBound(data.upper_bounds()[upper_bounds]))
            } else if lower_bounds < data.lower_bounds().len() {
                self.lower_bounds += 1;
                Some(Bound::LowerBound(data.lower_bounds()[lower_bounds]))
            } else {
                None
            }
        });

        match next_bound {
            Some(bound) => Poll::Ready(Some(bound)),
            None => {
                let () = ready!(self.check.block_on_inference_var(self.inference_var, cx));
                Poll::Ready(None)
            }
        }
    }
}

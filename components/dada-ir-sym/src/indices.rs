use salsa::Update;

use crate::ir::symbol::SymGenericKind;

/// Identifies a particular inference variable during type checking.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct InferVarIndex(usize);

/// Create an instance of `Self` from an inference variable
pub trait FromInfer<'db> {
    fn infer(db: &'db dyn crate::Db, var: InferVarIndex) -> Self;
}

impl InferVarIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for InferVarIndex {
    fn from(value: usize) -> Self {
        InferVarIndex(value)
    }
}

impl std::ops::Add<usize> for InferVarIndex {
    type Output = InferVarIndex;

    fn add(self, value: usize) -> Self {
        Self::from(self.as_usize().checked_add(value).unwrap())
    }
}

impl std::ops::Sub<InferVarIndex> for InferVarIndex {
    type Output = usize;

    fn sub(self, value: InferVarIndex) -> usize {
        self.as_usize().checked_sub(value.as_usize()).unwrap()
    }
}

/// Many of our types can be created from a variable
pub trait FromInferVar<'db> {
    fn infer(db: &'db dyn crate::Db, kind: SymGenericKind, var: InferVarIndex) -> Self;
}

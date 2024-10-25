use salsa::Update;

use crate::symbol::SymGenericKind;

/// Identifies a particular inference variable during type checking.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymInferVarIndex(usize);

impl SymInferVarIndex {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for SymInferVarIndex {
    fn from(value: usize) -> Self {
        SymInferVarIndex(value)
    }
}

impl std::ops::Add<usize> for SymInferVarIndex {
    type Output = SymInferVarIndex;

    fn add(self, value: usize) -> Self {
        Self::from(self.as_usize().checked_add(value).unwrap())
    }
}

impl std::ops::Sub<SymInferVarIndex> for SymInferVarIndex {
    type Output = usize;

    fn sub(self, value: SymInferVarIndex) -> usize {
        self.as_usize().checked_sub(value.as_usize()).unwrap()
    }
}

/// Many of our types can be created from a variable
pub trait FromInferVar<'db> {
    fn infer(db: &'db dyn crate::Db, kind: SymGenericKind, var: SymInferVarIndex) -> Self;
}

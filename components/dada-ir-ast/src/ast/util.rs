use std::ops::Deref;

use salsa::Update;

use crate::span::Span;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct AstVec<'db, T: Update> {
    //                    ------ FIXME: Bug in the derive?
    pub span: Span<'db>,
    pub values: Vec<T>,
}

impl<'db, T: Update> Deref for AstVec<'db, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

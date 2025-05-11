use std::ops::{Deref, DerefMut};

use salsa::Update;
use serde::Serialize;

use crate::span::Span;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub struct SpanVec<'db, T: Update> {
    //                    ------ FIXME: Bug in the derive?
    pub span: Span<'db>,
    pub values: Vec<T>,
}

impl<T: Update> Deref for SpanVec<'_, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T: Update> DerefMut for SpanVec<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<'db, T> IntoIterator for &'db SpanVec<'db, T>
where
    T: Update,
{
    type Item = &'db T;

    type IntoIter = std::slice::Iter<'db, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct TypedVec<I: TypedVecIndex, T> {
    data: Vec<T>,
    phantom: PhantomData<I>,
}

pub trait TypedVecIndex {
    fn into_usize(self) -> usize;
    fn from_usize(v: usize) -> Self;
}

impl<I: TypedVecIndex, T> Default for TypedVec<I, T> {
    fn default() -> Self {
        TypedVec {
            data: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<I: TypedVecIndex, T> From<Vec<T>> for TypedVec<I, T> {
    fn from(data: Vec<T>) -> Self {
        TypedVec {
            data,
            phantom: PhantomData,
        }
    }
}

impl<I: TypedVecIndex, T> From<TypedVec<I, T>> for Vec<T> {
    fn from(data: TypedVec<I, T>) -> Self {
        data.data
    }
}

impl<I: TypedVecIndex, T> TypedVec<I, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

impl<I: TypedVecIndex, T> std::ops::Index<I> for TypedVec<I, T> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index.into_usize()]
    }
}

impl<I: TypedVecIndex, T> std::ops::IndexMut<I> for TypedVec<I, T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index.into_usize()]
    }
}

impl<I: TypedVecIndex, T> Deref for TypedVec<I, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<I: TypedVecIndex, T> DerefMut for TypedVec<I, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<I: TypedVecIndex, T> Extend<T> for TypedVec<I, T> {
    fn extend<TI: IntoIterator<Item = T>>(&mut self, iter: TI) {
        self.data.extend(iter);
    }
}

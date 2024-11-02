use std::{ops::Deref, vec};

pub struct VecSet<T: Eq> {
    data: Vec<T>,
}

impl<T: Eq> VecSet<T> {
    pub fn new() -> Self {
        VecSet { data: Vec::new() }
    }

    pub fn insert(&mut self, item: T) -> bool {
        if !self.data.contains(&item) {
            self.data.push(item);
            true
        } else {
            false
        }
    }
}

impl<T: Eq> Default for VecSet<T> {
    fn default() -> Self {
        VecSet::new()
    }
}

impl<T: Eq> Deref for VecSet<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Eq> Extend<T> for VecSet<T> {
    fn extend<TI: IntoIterator<Item = T>>(&mut self, iter: TI) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<T: Eq> IntoIterator for VecSet<T> {
    type Item = T;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

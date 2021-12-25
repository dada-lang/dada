use std::{hash::Hash, marker::PhantomData};

use dada_collections::IndexVec;

/// An individual allocating table, where each thing
/// added to the table gets a unique index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocTable<K: salsa::AsId, V: Hash + Eq> {
    vec: IndexVec<salsa::Id, V>,
    phantom: PhantomData<K>,
}

impl<K: salsa::AsId, V: Hash + Eq> Default for AllocTable<K, V> {
    fn default() -> Self {
        Self {
            vec: IndexVec::default(),
            phantom: PhantomData,
        }
    }
}
impl<K: salsa::AsId, V: Hash + Eq> AllocTable<K, V> {
    pub fn add(&mut self, value: V) -> K {
        let index = self.vec.len();
        self.vec.push(value);
        let index: u32 = index.try_into().unwrap();
        K::from_id(salsa::Id::from_u32(index))
    }

    pub fn data(&self, key: K) -> &V {
        &self.vec[key.as_id()]
    }

    /// Replace the value for K with V.
    pub fn replace(&mut self, key: K, value: V) {
        self.vec[key.as_id()] = value;
    }
}

impl<K: salsa::AsId, V: Hash + Eq> std::ops::Index<K> for AllocTable<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.data(key)
    }
}

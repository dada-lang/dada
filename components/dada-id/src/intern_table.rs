use dada_collections::IndexSet;
use std::{hash::Hash, marker::PhantomData};

/// An individual interning table, where each unique thing added
/// to the table gets a unique index, but adding the same thing
/// twice gets the same index.
pub struct InternTable<K: salsa::AsId, V: Hash + Eq> {
    map: IndexSet<V>,
    phantom: PhantomData<K>,
}

impl<K: salsa::AsId, V: Hash + Eq> Default for InternTable<K, V> {
    fn default() -> Self {
        Self {
            map: IndexSet::default(),
            phantom: PhantomData,
        }
    }
}
impl<K: salsa::AsId, V: Hash + Eq> InternTable<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, value: V) -> K {
        let (index, _) = self.map.insert_full(value);
        let index: u32 = index.try_into().unwrap();
        K::from_id(salsa::Id::from_u32(index))
    }

    pub fn data(&self, key: K) -> &V {
        let index: usize = key.as_id().as_u32().try_into().unwrap();
        self.map.get_index(index).unwrap()
    }
}

impl<K: salsa::AsId, V: Hash + Eq> std::ops::Index<K> for InternTable<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.data(key)
    }
}

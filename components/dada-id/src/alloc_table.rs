use std::marker::PhantomData;

use dada_collections::IndexVec;

/// An individual allocating table, where each thing
/// added to the table gets a unique index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocTable<K: salsa::AsId, V> {
    vec: IndexVec<salsa::Id, V>,
    phantom: PhantomData<K>,
}

impl<K: salsa::AsId, V> Default for AllocTable<K, V> {
    fn default() -> Self {
        Self {
            vec: IndexVec::default(),
            phantom: PhantomData,
        }
    }
}
impl<K: salsa::AsId, V> AllocTable<K, V> {
    /// Returns the key that will be assigned to the next
    /// item that is added. As keys are assigned continguously,
    /// this also determines the range of valid keys
    /// (i.e., if this function returns N, then keys `0..N` are valid).
    pub fn next_key(&self) -> K {
        let index = self.vec.len();
        let index: u32 = index.try_into().unwrap();
        K::from_id(salsa::Id::from_u32(index))
    }

    pub fn add(&mut self, value: V) -> K {
        let key = self.next_key();
        self.vec.push(value);
        key
    }

    pub fn data(&self, key: K) -> &V {
        &self.vec[key.as_id()]
    }

    pub fn data_mut(&mut self, key: K) -> &mut V {
        &mut self.vec[key.as_id()]
    }

    /// Replace the value for K with V.
    pub fn all_keys(&self) -> impl Iterator<Item = K> {
        (0..self.vec.len()).map(|i| K::from_id(salsa::Id::from(i)))
    }
}

impl<K: salsa::AsId, V> std::ops::Index<K> for AllocTable<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.data(key)
    }
}

impl<K: salsa::AsId, V> std::ops::IndexMut<K> for AllocTable<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.data_mut(key)
    }
}

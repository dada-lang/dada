use dada_collections::IndexSet;
use std::{hash::Hash, marker::PhantomData};

/// Declares a struct usable as an interning id.
#[macro_export]
macro_rules! intern_id {
    ($v:vis struct $n:ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        $v struct $n(salsa::Id);

        impl salsa::AsId for $n {
            fn as_id(self) -> salsa::Id {
                self.0
            }

            fn from_id(id: salsa::Id) -> Self {
                Self(id)
            }
        }
    }
}

/// Declares a struct containing a group of interning tables, along with methods for accessing them.
#[macro_export]
macro_rules! intern_tables {
    ($vis:vis struct $n:ident {
        $(
            $f:ident: $k:ty => $v:ty,
        )*
    }) => {
        $vis struct $n {
            $(
                $f: dada_intern::InternTable<$k, $v>,
            )*
        }

        impl Default for $n {
            fn default() -> Self {
                Self {
                    $f: $(<dada_intern::InternTable<$k,$v>>::new(),)*
                }
            }
        }

        $(
            impl std::ops::Index<$k> for $n {
                type Output = $v;

                fn index(&self) -> &$v {
                    &self.$f
                }
            }
        )*
    }
}

/// An individual interning table.
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

    pub fn intern(&mut self, value: V) -> K {
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

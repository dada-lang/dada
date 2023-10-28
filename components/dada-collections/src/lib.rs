use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

pub use rustc_hash::FxHashMap as Map;
pub use rustc_hash::FxHashSet as Set;

pub type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub type IndexSet<V> = indexmap::IndexSet<V, BuildHasherDefault<FxHasher>>;

pub type IndexVec<K, V> = typed_index_collections::TiVec<K, V>;

mod bitset;
pub use bitset::*;

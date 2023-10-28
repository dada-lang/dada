use std::marker::PhantomData;

use rustc_hash::{FxHashMap, FxHashSet};

/// Defines sets of keys `K` for a range of locations `P`.
/// Used to define a set of variables at each control point in the graph, for example.
pub struct TypedBitSets<P, K> {
    positions: P,
    keys: K,

    bits: Vec<u64>,

    /// Conceptually this is the data structure we are modeling.
    data: PhantomData<FxHashMap<K, FxHashSet<K>>>,
}

pub trait BitSetKey: From<usize> + Into<usize> + Ord + Copy {}
impl<K> BitSetKey for K where K: From<usize> + Into<usize> + Ord + Copy {}

const BITS_PER_WORD: usize = 64;
const ALL_BITS: usize = std::usize::MAX;

struct Words(usize);

impl Words {
    pub fn from(n: impl Into<usize>) -> Self {
        let n: usize = n.into();
        Words((n + BITS_PER_WORD - 1) / BITS_PER_WORD)
    }
}

struct Mask {
    word: usize,
    mask: u64,
}

impl Mask {
    pub fn from(n: impl Into<usize>) -> Self {
        let n: usize = n.into();
        let word = n / BITS_PER_WORD;
        let mask = (n & ALL_BITS) as u64;
        Mask { word, mask }
    }
}

impl<P: BitSetKey, K: BitSetKey> TypedBitSets<P, K> {
    pub fn new(positions: P, keys: K) -> Self {
        let per_position_words = Words::from(positions);
        let total_words = per_position_words.0 * keys.into();
        Self {
            positions,
            keys,
            bits: vec![0; total_words],
            data: PhantomData,
        }
    }

    /// Insert `key` at `position`, returning true if it was not already present.
    pub fn insert(&mut self, position: P, key: K) -> bool {
        assert!(position < self.positions);
        assert!(key < self.keys);
        let per_position_words = Words::from(self.positions);
        let position: usize = position.into() * per_position_words.0;
        let mask = Mask::from(key);
        let word = position + mask.word;
        let old_value = self.bits[word];
        let new_value = old_value | mask.mask;
        self.bits[word] = new_value;
        old_value != new_value
    }

    /// True if `position` contains `key`.
    pub fn contains(&self, position: P, key: K) -> bool {
        assert!(position < self.positions);
        assert!(key < self.keys);
        let per_position_words = Words::from(self.positions);
        let position: usize = position.into() * per_position_words.0;
        let mask = Mask::from(key);
        let word = position + mask.word;
        let old_value = self.bits[word];
        (old_value & mask.mask) != 0
    }

    /// Inserts all bits set at `position_source` into `position_target`,
    /// returning true if any new bits were added.
    pub fn insert_all(&mut self, position_target: P, position_source: P) -> bool {
        assert!(position_target < self.positions);
        assert!(position_source < self.positions);

        if position_target == position_source {
            return false;
        }

        let per_position_words = Words::from(self.positions);
        let start_target = position_target.into() * per_position_words.0;
        let start_source = position_source.into() * per_position_words.0;

        let mut changed = false;
        for i in 0..per_position_words.0 {
            let old_value = self.bits[start_target + i];
            let new_value = old_value | self.bits[start_source + i];
            self.bits[start_target + i] = new_value;
            changed |= old_value != new_value;
        }

        changed
    }
}

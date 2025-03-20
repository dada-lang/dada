use std::{ops::Deref, vec};

use salsa::Update;
use serde::Serialize;

/// A set of elements, stored in a sorted vector.
///
/// This is more efficient than a `HashSet` for small sets, and allows for
/// efficient iteration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct VecSet<T: Ord> {
    /// Elements in the set, always sorted.
    sorted_elements: Vec<T>,
}

unsafe impl<T: Update + Ord> salsa::Update for VecSet<T> {
    unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
        unsafe {
            Update::maybe_update(
                &mut (*old_pointer).sorted_elements,
                new_value.sorted_elements,
            )
        }
    }
}

impl<T: Ord> VecSet<T> {
    pub fn new() -> Self {
        VecSet {
            sorted_elements: Vec::new(),
        }
    }

    pub fn singleton(item: T) -> Self {
        VecSet {
            sorted_elements: vec![item],
        }
    }

    /// Insert `item` into the set.
    ///
    /// Returns `true` if the item was not already in the set.
    pub fn insert(&mut self, item: T) -> bool {
        match self.sorted_elements.binary_search(&item) {
            Ok(_) => false,
            Err(idx) => {
                self.sorted_elements.insert(idx, item);
                true
            }
        }
    }

    /// Extend the set with the items from `other`.
    pub fn extend(&mut self, other: impl IntoIterator<Item = T>) {
        self.sorted_elements.extend(other);
        self.sorted_elements.sort_unstable();
        self.sorted_elements.dedup();
    }

    /// Check if the set contains `item`.
    pub fn contains(&self, item: &T) -> bool {
        self.sorted_elements.binary_search(item).is_ok()
    }

    /// Get the number of elements in the set.
    pub fn len(&self) -> usize {
        self.sorted_elements.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.sorted_elements.is_empty()
    }
}

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        VecSet::new()
    }
}

impl<T: Ord> Deref for VecSet<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.sorted_elements
    }
}

impl<T: Ord> Extend<T> for VecSet<T> {
    fn extend<TI: IntoIterator<Item = T>>(&mut self, iter: TI) {
        self.sorted_elements.extend(iter);
        self.sorted_elements.sort_unstable();
        self.sorted_elements.dedup();
    }
}

impl<T: Ord> IntoIterator for VecSet<T> {
    type Item = T;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.sorted_elements.into_iter()
    }
}

impl<'e, T: Ord> IntoIterator for &'e VecSet<T> {
    type Item = &'e T;

    type IntoIter = std::slice::Iter<'e, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sorted_elements.iter()
    }
}

impl<T: Ord> From<T> for VecSet<T> {
    fn from(value: T) -> Self {
        VecSet::singleton(value)
    }
}

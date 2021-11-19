use std::collections::btree_set::IntoIter;
use std::collections::BTreeSet;

/// A set of orderable distinct elements with a maximum size.
pub struct MinSet<T: Ord> {
    btree: BTreeSet<T>,
    max_len: usize,
}

impl<T: Ord> MinSet<T> {
    /// Create a new set with at most `max_len` elements.
    pub fn new(max_len: usize) -> Self {
        MinSet {
            btree: BTreeSet::new(),
            max_len: max_len,
        }
    }

    /// Insert a new `item` to the set.
    /// If the set overflows, the maximum item is deleted.
    /// If an other equal item is already in the set, then nothing is
    /// done and the method returns `false`, else it returns `true`.
    pub fn push(&mut self, item: T) -> bool {
        if !self.btree.insert(item) {
            false
        } else {
            if self.btree.len() > self.max_len {
                let mut btree =
                    std::mem::replace(&mut self.btree, BTreeSet::new())
                        .into_iter();
                btree.next();
                for e in btree {
                    self.btree.insert(e);
                }
            };
            true
        }
    }

    /// Delete the set and iterate items inside it.
    pub fn into_iter(self) -> IntoIter<T> {
        self.btree.into_iter()
    }
}

use std::collections::btree_set::IntoIter;
use std::collections::BTreeSet;

/// A set of orderable distinct elements with a maximum size.
pub struct KMin<T: Ord> {
    btree: BTreeSet<T>,
    max_len: usize,
}

impl<T: Ord> KMin<T> {
    /// Create a new set with at most `max_len` elements.
    pub fn new(k: usize) -> Self {
        KMin {
            btree: BTreeSet::new(),
            max_len: k,
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
                    std::mem::take(&mut self.btree).into_iter();
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

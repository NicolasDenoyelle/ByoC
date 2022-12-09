use std::collections::btree_set::IntoIter;
use std::collections::BTreeSet;

/// A set of orderable distinct elements with a maximum size.
pub struct KMin<T: Ord> {
    pub(super) btree: BTreeSet<(T, usize)>,
    max_size: usize,
    pub(super) size: usize,
}

impl<T: Ord> KMin<T> {
    /// Create a new set with at most `max_len` elements.
    pub fn new(k: usize) -> Self {
        KMin {
            btree: BTreeSet::new(),
            max_size: k,
            size: 0usize,
        }
    }

    /// Insert a new `item` to the set.
    /// If the set overflows, the maximum item is deleted.
    /// If an other equal item is already in the set, then nothing is
    /// done and the method returns `false`, else it returns `true`.
    pub fn push(&mut self, item: T, item_size: usize) -> bool {
        if !self.btree.insert((item, item_size)) {
            false
        } else {
            self.size += item_size;
            if self.size - item_size < self.max_size {
                return true;
            }

            let mut iter = std::mem::take(&mut self.btree).into_iter();

            loop {
                let (item, item_size) = match iter.next() {
                    None => break,
                    Some(s) => s,
                };

                if (self.size - item_size) < self.max_size {
                    self.btree.insert((item, item_size));
                    break;
                }
                self.size -= item_size;
            }
            for e in iter {
                self.btree.insert(e);
            }
            true
        }
    }

    /// Delete the set and iterate items inside it.
    pub fn into_iter(self) -> IntoIter<(T, usize)> {
        self.btree.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::KMin;

    #[test]
    fn test_invariants() {
        let mut kmin = KMin::new(5);
        assert!(kmin.push(2, 4));

        // Overflow by 3. It is ok on first overflow.
        assert!(kmin.push(3, 4));
        assert_eq!(kmin.size, 8);

        // Overflow by one more.
        // This should pop the smallest element and keep at least max_size
        // inside the container.
        assert!(kmin.push(4, 1));
        assert!(!kmin.btree.contains(&(2, 4)));
        assert_eq!(kmin.size, 5);

        // Overflow by one.
        // The smallest element (3,4) is not popped because the size in the
        // container would fall below target `max_size`.
        assert!(kmin.push(5, 1));
        assert!(kmin.btree.contains(&(3, 4)));
        assert_eq!(kmin.size, 6);

        // Overflow by one more.
        // The newly inserted element is the max element.
        // It can be inserted while keeping size >= max_size.
        assert!(kmin.push(1, 1));
        assert!(!kmin.btree.contains(&(1, 1)));
        assert_eq!(kmin.size, 6);
    }

    #[test]
    fn test_pop_1() {
        let mut kmin = KMin::new(1);
        kmin.push(0, 2);
        kmin.push(2, 2);
        kmin.push(1, 2);
        assert_eq!(kmin.btree.len(), 1);
        assert!(kmin.btree.contains(&(2, 2)));
    }
}

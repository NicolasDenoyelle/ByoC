use super::BTree;
use crate::utils::size::find_cut_at_size;
use crate::BuildingBlock;
use std::collections::BTreeMap;
use std::rc::Rc;

impl<K, V> BuildingBlock<K, V> for BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the size set by the constructor
    /// [`BTree::new()`](struct.BTree.html#method.new).
    /// The meaning of this methods depends on the meaning of the
    /// `elements_size()` function that can be set with the method
    /// [`with_element_size()`](struct.struct..html#method.with_element_size).
    /// For instance, capacity can be the number of elements in the BTree
    /// when all elements size is one, or it can be the maximum stack
    /// size when elements size is the size of the element on the stack.
    fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the size currently occupied by elements in this [`BTree`].
    ///
    /// This is the sum of this [`BTree`] elements size, as defined by the
    /// function `element_size()` set by the method
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    fn size(&self) -> usize {
        self.total_size
    }

    type FlushIterator = std::iter::Map<
        std::collections::btree_map::IntoIter<K, Rc<V>>,
        fn((K, Rc<V>)) -> (K, V),
    >;
    fn flush(&mut self) -> Self::FlushIterator {
        let mut elements = BTreeMap::new();
        std::mem::swap(&mut elements, &mut self.map);
        self.set.clear();
        self.total_size = 0;

        elements.into_iter().map(|(k, rc)| (k, Self::as_value(rc)))
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Insert key/value pairs in the container.
    ///
    /// The total `size` of the elements to push is computed using
    /// `element_size()` function set by the method
    /// [`with_element_size()`](struct.BTree.html#method.with_element_size).
    ///
    /// If the room in the container is sufficient to store every `elements`,
    /// then they are inserted and an empty vector is returned.
    ///
    /// If the `size` is larger than or equal to this [`BTree`] capacity, then
    /// at least, all the elements contained in the [`BTree`] will be returned.
    /// Additionally, if `size` is strictly larger than this container size, then
    /// the `elements` with the greatest values that don't fit in are also
    /// returned.
    ///
    /// Otherwise, the redundant keys that were in the container will be
    /// returned. Additionally, if there are more `elements` to insert than the
    /// remaining room, enough existing elements inside the [`BTree`] will be
    /// popped to make room for the new `elements`.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let size: usize = elements
            .iter()
            .map(|(k, v)| (self.element_size)((k, v)))
            .sum();

        // If we insert more than available capacity, we need to flush
        // and return flushed plus extra exceeding capacity.
        if size >= self.capacity {
            let mut out: Vec<(K, V)> = self.flush().collect();
            self.insert_values_unchecked(elements);
            self.total_size = size;
            if size > self.capacity {
                out.append(&mut self.pop(size - self.capacity));
            }
            return out;
        }

        // Remove redundant keys.
        let mut out = self.take_multiple(
            &mut elements.iter().map(|(k, _)| *k).collect(),
        );

        // Get the most we can fit in.
        let room = self.capacity - self.total_size;

        if room < size {
            out.append(&mut self.pop(size - room));
        }

        self.insert_values_unchecked(elements);
        self.total_size += size;
        out
    }

    /// Free up to `size` space from the container.
    ///
    /// If the [`BTree`] is empty, an empty vector is returned.
    ///
    /// If the `size` to pop is less than the current size in the container,
    /// the container is emptied and all its elements are returned.
    ///
    /// Otherwise, the elements with the greatest values are evicted, until
    /// the sum of there sizes is at least equal to the requested `size` to
    /// evict.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        if self.map.is_empty() {
            return Vec::new();
        }
        if self.total_size <= size {
            return self.flush().collect();
        }

        let (_, cut_size, option_rc_k) = find_cut_at_size(
            &self.set,
            |(rc, k)| (self.element_size)((k, rc)),
            size,
        );

        let e = option_rc_k.map(|(rc, k)| (Rc::clone(rc), *k)).expect(
            "Failure in find_cut_at_size(). It should not return None.",
        );

        self.total_size -= cut_size;
        let out_set = self.set.split_off(&e);
        drop(e);
        let mut out = Vec::with_capacity(out_set.len());
        for (rc, k) in out_set.into_iter() {
            // Drop Rc<V> clone to avoid panicking when unwrapping the
            // value below
            drop(rc);
            let rc = self.map.remove(&k).unwrap();
            out.push((k, Self::as_value(rc)));
        }
        out
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.map.remove(key) {
            None => None,
            Some(rc) => {
                self.total_size -= (self.element_size)((key, rc.as_ref()));
                self.set.remove(&(rc.clone(), *key));
                Some((*key, Self::as_value(rc)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::{test_building_block, TestElement};

    #[test]
    fn building_block_default() {
        test_building_block(BTree::new(0), true);
        test_building_block(BTree::new(10), true);
        test_building_block(BTree::new(100), true);
    }

    #[test]
    fn building_block_stack_size() {
        test_building_block(
            BTree::new(0)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
        test_building_block(
            BTree::new(10)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
        test_building_block(
            BTree::new(100)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
    }
}

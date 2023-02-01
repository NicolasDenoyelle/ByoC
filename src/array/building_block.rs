use super::Array;
use crate::utils::size::find_cut_at_size;
use crate::BuildingBlock;

impl<'a, K, V> BuildingBlock<'a, K, V> for Array<(K, V)>
where
    K: 'a + Ord,
    V: 'a + Ord,
{
    /// Get the maximum storage size of this [`Array`].
    ///
    /// This is the size set by the constructor
    /// [`Array::new()`](struct.Array.html#method.new).
    /// The meaning of this methods depends on the meaning of the
    /// elements size that can be set with the method
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    /// For instance, capacity can be the number of elements in the array
    /// when all elements size is one, or it can be the maximum stack
    /// size when elements size is the size of the element on the stack.
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.total_size = 0;
        Box::new(self.values.split_off(0).into_iter())
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(k, _)| k == key)
    }

    /// Get the size currently occupied by elements in this [`Array`].
    ///
    /// This is the sum of this [`Array`] elements size, as defined by the
    /// function `element_size()` set by the method
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    fn size(&self) -> usize {
        self.total_size
    }

    /// Free up to `size` space from the container.
    ///
    /// If the [`Array`] is empty, an empty vector is returned.
    ///
    /// If the `size` to pop is greater than the current size in the container,
    /// the container is emptied and all its elements are returned.
    ///
    /// Otherwise, array elements are sorted by value using values
    /// [`std::cmp::Ord`] trait and the greatest ones are evicted until the
    /// sum of evicted elements' size meets the `size` threshold. Elements
    /// size is computed with the function `element_size()` set by the method
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        // If the vector is empty there is nothing to return.
        if self.values.is_empty() {
            return Vec::new();
        }

        // If the vector has less total size than requested we return
        // everything.
        if self.total_size <= size {
            self.total_size = 0;
            return self.values.split_off(0);
        }

        // Otherwise, we need to evict the greatest elements.

        // Sort values.
        self.values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));

        // Find cut.
        let (cut, cut_size, _) =
            find_cut_at_size(&self.values, self.element_size, size);

        // Evict.
        self.total_size -= cut_size;
        self.values.split_off(cut)
    }

    /// Insert key/value pairs in the container.
    ///
    /// The total `size` of the elements to push is computed using
    /// `element_size()` function set by the method
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    ///
    /// If the room in the container is sufficient to store every `elements`,
    /// then they are inserted and an empty vector is returned.
    ///
    /// If the `size` is larger than or equal to this [`Array`] capacity, then
    /// at least, all the elements contained in the [`Array`] will be returned.
    /// Additionally, if `size` is strictly larger than this container size,
    /// then the `elements` with the greatest values that don't fit in are also
    /// returned.
    ///
    /// Otherwise, the redundant keys that were in the container will be
    /// returned. Additionally, if there are more `elements` to insert than the
    /// remaining room, enough existing elements inside the [`Array`] will be
    /// popped to make room for the new `elements`.
    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let size: usize =
            elements.iter().map(|e| (self.element_size)(e)).sum();
        let room = self.capacity - self.total_size;

        if size <= room {
            self.total_size += size;
            self.values.append(&mut elements);
            return elements;
        }

        if size == self.capacity {
            std::mem::swap(&mut self.values, &mut elements);
            self.total_size = size;
            return elements;
        }

        if size > self.capacity {
            elements.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));
            let (cut, cut_size, _) = find_cut_at_size(
                &elements,
                self.element_size,
                size - self.capacity,
            );
            let mut out = elements.split_off(cut);
            std::mem::swap(&mut self.values, &mut elements);
            elements.append(&mut out);
            self.total_size = size - cut_size;
            return elements;
        }

        let out = self.pop(size - room);
        self.total_size += size;
        self.values.append(&mut elements);
        out
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.values.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i)
            } else {
                None
            }
        }) {
            None => None,
            Some(i) => {
                self.total_size -=
                    (self.element_size)(self.values.get(i).unwrap());
                Some(self.values.swap_remove(i))
            }
        }
    }

    /// Take multiple keys out the container at once.
    ///
    /// This method is an optimization of `take()` method for removal of
    /// multiple keys. The method first sorts the input keys. Then, it walks the
    /// [`Array`] elements one by one in reverse order, and searches for a
    /// matching key in the input array of keys. When a key is matched, it is
    /// removed from the [`Array`] and appended to the returned elements.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());
        keys.sort();
        for i in (0..self.values.len()).rev() {
            if let Ok(j) = keys.binary_search(&self.values[i].0) {
                keys.remove(j);
                self.total_size -=
                    (self.element_size)(self.values.get(i).unwrap());
                ret.push(self.values.swap_remove(i));
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::tests::{test_building_block, TestElement};

    #[test]
    fn building_block_default() {
        test_building_block(Array::new(0), true);
        test_building_block(Array::new(10), true);
        test_building_block(Array::new(100), true);
    }

    #[test]
    fn building_block_stack_size() {
        test_building_block(
            Array::new(0)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
        test_building_block(
            Array::new(10)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
        test_building_block(
            Array::new(100)
                .with_element_size(|_| std::mem::size_of::<TestElement>()),
            true,
        );
    }
}

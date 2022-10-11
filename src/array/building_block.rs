use super::Array;
use crate::BuildingBlock;

impl<'a, K, V> BuildingBlock<'a, K, V> for Array<(K, V)>
where
    K: 'a + Ord,
    V: 'a + Ord,
{
    /// Get the maximum "size" that elements in the container can fit.
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

    fn size(&self) -> usize {
        self.total_size
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned array contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// sorting the array on values and spitting it where appropriate.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
        let i = self.values.len();
        let out = self.values.split_off(i - std::cmp::min(i, n));
        let out_size: usize =
            out.iter().map(|e| (self.element_size)(e)).sum();
        self.total_size -= out_size;
        out
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
    /// returned.
    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut i = 0;
        for e in elements.iter() {
            let size = (self.element_size)(e);
            if self.total_size + size > self.capacity {
                break;
            }
            self.total_size += size;
            i += 1;
        }

        let out = elements.split_off(i);
        if i > 0 {
            self.values.append(&mut elements);
        }
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

    // One pass take
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

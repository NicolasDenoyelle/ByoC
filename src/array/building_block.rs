use super::Array;
use crate::BuildingBlock;

impl<'a, K, V> BuildingBlock<'a, K, V> for Array<(K, V)>
where
    K: 'a + Ord,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.values.split_off(0).into_iter())
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(k, _)| k == key)
    }

    fn count(&self) -> usize {
        self.values.len()
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
        self.values.split_off(i - std::cmp::min(i, n))
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
    /// returned.
    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = std::cmp::min(
            self.capacity - self.values.len(),
            elements.len(),
        );
        let out = elements.split_off(n);

        if n > 0 {
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
            Some(i) => Some(self.values.swap_remove(i)),
        }
    }

    // One pass take
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());
        keys.sort();
        for i in (0..self.values.len()).rev() {
            if let Ok(j) = keys.binary_search(&self.values[i].0) {
                keys.remove(j);
                ret.push(self.values.swap_remove(i));
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        test_building_block(Array::new(0));
        test_building_block(Array::new(10));
        test_building_block(Array::new(100));
    }
}

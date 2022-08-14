use super::Multilevel;
use crate::{BuildingBlock, Prefetch};

impl<'a, K, V, L, R> Prefetch<'a, K, V> for Multilevel<K, V, L, R>
where
    K: 'a + Ord,
    V: 'a,
    L: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
    R: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
{
    /// Multilevel prefetch implementation moves matching keys
    /// in the right side into the left side.
    /// This is achieved by calling the
    /// [`push()`](struct.Multilevel.html#method.push) method after retrieving
    /// elements from the right side.
    fn prefetch(&mut self, mut keys: Vec<K>) {
        // Then right side.
        let matches = self.right.take_multiple(&mut keys);

        // Finally insert matches.
        // Reinsertion must work because we the container still has the same
        // number of elements.
        if !matches.is_empty() {
            assert!(self.push(matches).pop().is_none());
        }
    }

    /// This method will take matching keys on the left side then on
    /// the right side.
    /// Matching keys found on the left side are not searched on the right
    /// side.
    /// Input `keys` is updated as a side effect to contain
    /// only non matching keys.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.sort();

        let mut left = self.left.take_multiple(keys);

        // Remove matches from keys before querying on the right side.
        for (k, _) in left.iter() {
            if let Ok(i) = keys.binary_search(k) {
                keys.remove(i);
            }
        }

        let mut right = self.right.take_multiple(keys);

        // Remove matching keys in case these keys are used in other
        // calls to take_multiple.
        for (k, _) in right.iter() {
            if let Ok(i) = keys.binary_search(k) {
                keys.remove(i);
            }
        }

        // Return final matches.
        left.append(&mut right);
        left
    }
}

#[cfg(test)]
mod tests {
    use super::Multilevel;
    use crate::tests::test_prefetch;
    use crate::Array;
    #[test]
    fn prefetch() {
        test_prefetch(Multilevel::new(Array::new(0), Array::new(0)));
        test_prefetch(Multilevel::new(Array::new(0), Array::new(10)));
        test_prefetch(Multilevel::new(Array::new(10), Array::new(0)));
        test_prefetch(Multilevel::new(Array::new(10), Array::new(100)));
    }
}

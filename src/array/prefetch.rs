pub use super::Array;
use crate::Prefetch;

impl<'a, K: 'a + Ord, V: 'a + Ord> Prefetch<'a, K, V> for Array<(K, V)> {
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
    use crate::tests::test_prefetch;
    #[test]
    fn prefetch() {
        test_prefetch(Array::new(0));
        test_prefetch(Array::new(10));
        test_prefetch(Array::new(100));
    }
}

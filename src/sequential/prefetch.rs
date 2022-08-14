use super::Sequential;
use crate::{BuildingBlock, Prefetch};

impl<'a, K, V, C> Prefetch<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        let _ = self.lock.lock_for(()).unwrap();
        let mut container = self.container.as_mut();
        container.prefetch(keys)
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let _ = self.lock.lock_for(()).unwrap();
        let mut container = self.container.as_mut();
        container.take_multiple(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::tests::test_prefetch;
    use crate::Array;

    #[test]
    fn prefetch() {
        test_prefetch(Sequential::new(Array::new(0)));
        test_prefetch(Sequential::new(Array::new(100)));
    }
}

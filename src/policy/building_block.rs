use crate::policy::{Ordered, Reference, ReferenceFactory};
use crate::BuildingBlock;
use crate::Policy;

impl<'a, K, V, C, R, F> BuildingBlock<'a, K, V> for Policy<C, V, R, F>
where
    K: 'a + Ord,
    V: 'a,
    R: 'a + Reference<V>,
    C: Ordered<R> + BuildingBlock<'a, K, R>,
    F: ReferenceFactory<V, R>,
{
    fn capacity(&self) -> usize {
        self.container.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.container.flush().map(|(k, r)| (k, r.unwrap())))
    }

    fn count(&self) -> usize {
        self.container.count()
    }

    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.container.take(key).map(|(k, r)| (k, r.unwrap()))
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        self.container
            .take_multiple(keys)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.container
            .pop(n)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let (container, factory) =
            (&mut self.container, &mut self.factory);
        container
            .push(
                elements
                    .into_iter()
                    .map(|(k, v)| (k, factory.wrap(v)))
                    .collect(),
            )
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::policy::Default;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(Policy::new(Array::new(i), Default {}));
        }
    }
}
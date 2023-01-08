use super::FlushStopper;
use crate::BuildingBlock;

impl<'a, K, V, C> BuildingBlock<'a, K, V> for FlushStopper<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.container.capacity()
    }
    fn size(&self) -> usize {
        self.container.size()
    }
    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.container.take(key)
    }
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        self.container.take_multiple(keys)
    }
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        self.container.pop(size)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        self.container.push(values)
    }
    /// Returns an empty iterator.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(std::iter::empty())
    }
}

use super::{Decoration, DecorationFactory};
use crate::BuildingBlock;
use crate::Decorator;

impl<K, V, C, F> BuildingBlock<K, V> for Decorator<C, V, F>
where
    K: Ord,
    C: BuildingBlock<K, F::Item>,
    F: DecorationFactory<V>,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the capacity of the container wrapped in this [`Decorator`]
    /// container.
    fn capacity(&self) -> usize {
        self.container.capacity()
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the size of the container wrapped in this [`Decorator`]
    /// container.
    fn size(&self) -> usize {
        self.container.size()
    }

    /// Check if container contains a matching key.
    ///
    /// This calls and returns the value of the decorated container
    /// [`contains()`](trait.BuildingBlock.html#method.contains)
    /// method.
    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// This calls and returns the value of the decorated container
    /// [`take()`](trait.BuildingBlock.html#method.take)
    /// method and will remove the decoration from the taken value before
    /// returning it.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.container.take(key).map(|(k, r)| (k, r.unwrap()))
    }

    /// Take multiple keys out of a container at once.
    ///
    /// This calls and returns the value of the decorated container
    /// [`take_multiple()`](trait.BuildingBlock.html#method.take_multiple)
    /// method and will remove the decoration from the taken values before
    /// returning them.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        self.container
            .take_multiple(keys)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }

    /// Free up to `size` space from the container.
    ///
    /// This calls and returns the value of the decorated container
    /// [`pop()`](trait.BuildingBlock.html#method.pop)
    /// method on decorated values. It will remove the decoration from the
    /// evicted values before returning them.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.container
            .pop(n)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }

    /// Insert key/value pairs in the container.
    ///
    /// This calls and returns the value of the decorated container
    /// [`push()`](trait.BuildingBlock.html#method.push)
    /// method. Inserted values will be decorated by the container before
    /// insertion.
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

    type FlushIterator =
        std::iter::Map<C::FlushIterator, fn((K, F::Item)) -> (K, V)>;

    /// Empty the container and retrieve all of its elements.
    ///
    /// This calls and returns the value of the decorated container
    /// [`flush()`](trait.BuildingBlock.html#method.flush)
    /// method and will remove the decoration from the flushed value to return
    /// on each iteration.
    fn flush(&mut self) -> Self::FlushIterator {
        self.container.flush().map(|(k, r)| (k, r.unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::Decorator;
    use crate::decorator::Default;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(
                Decorator::new(Array::new(i), Default {}),
                true,
            );
        }
    }
}

use crate::BuildingBlock;

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html) to accelerate next lookups
/// for a set of keys.
///
/// This trait provides a
/// [`prefetch()`](trait.Prefetch.html#method.prefetch) method allowing
/// to pay some compute cost upfront to accelerate the next lookup of some
/// keys. This is usefull in a context where a thread runs in the background
/// to reorganize a building block while the building block user is busy
/// doing something else. This feature is also aimed for future
/// implementations of an automatic prefetcher that can predict next
/// accessed keys and prefetch them in the background.
pub trait Prefetch<'a, K: 'a, V: 'a>: BuildingBlock<'a, K, V> {
    /// This the method that will reorganize the building block to
    /// accelerate next lookup of some `keys`. The default implementation
    /// does nothing.
    fn prefetch(&mut self, _keys: Vec<K>) {}

    /// Optimized implementation to take multiple keys out of a building
    /// block. This method returns a vector of all elements matching input
    /// `keys` that were inside a building block. Input keys can be
    /// altered only to remove keys that have been taken out of the
    /// building block.
    /// This method is aimed to accelerate the implementation of
    /// [`prefetch()`](trait.Prefetch.html#method.prefetch) method.
    /// BuildingBlock implementer should make sure to implement this method
    /// if it can be faster than the default implementation.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.iter().filter_map(|k| self.take(k)).collect()
    }
}

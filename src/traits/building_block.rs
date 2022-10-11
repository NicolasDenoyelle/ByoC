/// This is the main abstraction: the Building block interface for
/// key/value storage.
///
/// `BuildingBlock` trait defines the primitives to build a
/// data processing pipeline implementing a key/value data container.
/// The interface is made such that `BuildingBlock` implementers can be
/// assembled in a pipeline fashion to build a container that will meet
/// features and performance requirement of users key/value access
/// workloads.
///
/// A typical key/value container implementation could be a cache
/// with multiple layers of increasing size and decreasing performance,
/// with an eviction policy such that most accessed data live in the
/// fastest layer.
///
/// See
/// [`BuildingBlock` implementors](trait.BuildingBlock.html#implementors)
/// for more details on structuring building blocks together.
pub trait BuildingBlock<'a, K: 'a, V: 'a> {
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This value emulate the container maximum memory footprint.
    /// The real meaning of this value depends on the implementation.
    /// For instance, it could represent the maximum disk footprint of the file
    /// where elements are stored, or the maximum memory footprint of elements
    /// in device memory.
    ///
    /// As a rule of thumbs,
    /// adding elements to a container should be O(n) in space complexity
    /// while metadata should occupy O(1) (or something negligible,
    /// e.g O(log(n))) space complexity. At any point in time, the
    /// [`size()`](trait.BuildingBlock.html#tymethod.size) should be less or
    /// equal than [`capacity()`](trait.BuildingBlock.html#tymethod.capacity).
    fn capacity(&self) -> usize;

    /// Get the "size" occupied by this container.
    ///
    /// This value emulate the container memory footprint.
    /// The real meaning of this value depends on the implementation.
    /// Here the footprint may decouple the size of container metadata,
    /// versus the footprint of elements where they are stored, to only
    /// report the latter.
    ///
    /// This is hard to test because it depends on the medium where the
    /// container is stored. We may expect that insertion without overflow
    /// are going to increase container size, that removal of elements decreases
    /// it and that container [`size()`](trait.BuildingBlock.html#tymethod.size)
    /// will never exceed its
    /// [`capacity()`](trait.BuildingBlock.html#tymethod.capacity).
    fn size(&self) -> usize;

    /// Check if container contains a matchig key.
    fn contains(&self, key: &K) -> bool;

    /// Take the matching key/value pair out of the container.
    fn take(&mut self, key: &K) -> Option<(K, V)>;

    /// Optimized implementation to take multiple keys out of a building
    /// block. This method returns a vector of all elements matching input
    /// `keys` that were inside a building block. Input keys can be
    /// altered only to remove keys that have been taken out of the
    /// building block.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.iter().filter_map(|k| self.take(k)).collect()
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// The eviction policy deciding which elements are popped out is
    /// implementation defined.
    /// Implementations also implementing the marker trait
    /// [`Ordered`](policy/trait.Ordered.html) will guarantee the eviction
    /// of elements with the largest value. Usually, such building block
    /// are meant to be wrapped into a
    /// [`Policy`](struct.Policy.html)
    /// `BuildingBlock` to define the eviction policy.
    fn pop(&mut self, n: usize) -> Vec<(K, V)>;

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    /// The trait does not make a contract on what is returned.
    /// It could be for instance the values not fitting in the container or
    /// some values from the container depending on trade-offs
    /// or desired behavior.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a>;
}

impl<'a, K: 'a, V: 'a> BuildingBlock<'a, K, V>
    for Box<dyn BuildingBlock<'a, K, V> + 'a>
{
    fn capacity(&self) -> usize {
        (**self).capacity()
    }
    fn size(&self) -> usize {
        (**self).size()
    }
    fn contains(&self, key: &K) -> bool {
        (**self).contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        (**self).take(key)
    }
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        (**self).pop(n)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        (**self).push(values)
    }
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        (**self).flush()
    }
}

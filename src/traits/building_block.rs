/// This is the main abstraction: the Building block interface for
/// key/value storage.
///
/// `BuildingBlock` trait defines the primitives to build a
/// key/value data storage pipeline of containers.
/// The interface is made such that `BuildingBlock` implementers can be
/// assembled in a pipelined fashion to build a container that will meet
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
pub trait BuildingBlock<K, V> {
    /// The type of the iterator yielded by
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method.
    ///
    /// Each implementation of this trait may have a specific iterator type.
    type FlushIterator: Iterator<Item = (K, V)>;

    /// Get the maximum storage size of this [`BuildingBlock`].
    ///
    /// This value emulate the container maximum memory footprint.
    /// The real meaning of this value depends on the implementation.
    /// For instance, it could represent the maximum disk footprint of the file
    /// where elements are stored, or the maximum memory footprint of elements
    /// in a device memory.
    ///
    /// At any point in time, the
    /// [`size()`](trait.BuildingBlock.html#tymethod.size) should be less or
    /// equal to [`capacity()`](trait.BuildingBlock.html#tymethod.capacity).
    fn capacity(&self) -> usize;

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This value emulate the container memory footprint.
    /// The real meaning of this value depends on the implementation.
    /// Getting a real memory footprint may be hard. It depends on the
    /// medium where the container is stored. We may expect that insertion
    /// without overflow are going to increase container size and that removal
    /// of elements decreases it.
    ///
    /// It is compulsory that a container
    /// [`size()`](trait.BuildingBlock.html#tymethod.size) will never exceed its
    /// [`capacity()`](trait.BuildingBlock.html#tymethod.capacity).
    fn size(&self) -> usize;

    /// Check if container contains a matching key.
    fn contains(&self, key: &K) -> bool;

    /// Take the matching key/value pair out of the container.
    ///
    /// After this method is called, if the key was present in the container,
    /// calling [`contains()`](trait.BuildingBlock.html#tymethod.contains) on
    /// the same key should return `false` if the container does not store
    /// duplicate keys.
    fn take(&mut self, key: &K) -> Option<(K, V)>;

    /// Take multiple keys out of a container at once.
    ///
    /// This intended to optimize the process of taking multiple keys out of a
    /// building block, compared to using
    /// [`take()`](trait.BuildingBlock.html#tymethod.take) method repeatedly.
    ///
    /// This method returns a vector of all elements matching input `keys` that
    /// were inside a building block. Input `keys` vector may be altered to
    /// remove keys that have been taken out of the building block. This is used
    /// when forwarding the `take_multiple()` request to multiple building
    /// blocks, avoiding query a found key multiple times.
    ///
    /// As for [`take()`](trait.BuildingBlock.html#tymethod.take), for every key
    /// taken out of the container,
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains) is expected
    /// to return `false` if the container does not store duplicate keys.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.iter().filter_map(|k| self.take(k)).collect()
    }

    /// Free up to `size` space from the container.
    ///
    /// The meaning of `size should be the same as for
    /// [`size()`](trait.BuildingBlock.html#tymethod.size) and
    /// [`capacity()`](trait.BuildingBlock.html#tymethod.capacity) methods.
    ///
    /// If the container [`size()`](trait.BuildingBlock.html#tymethod.size) is
    /// greater than `size` argument, at least `size` space needs to be freed
    /// by returning elements occupying that space.
    /// If less than `size` space is occupied by values in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    ///
    /// The eviction policy deciding which elements are popped out is
    /// implementation defined.
    fn pop(&mut self, size: usize) -> Vec<(K, V)>;

    /// Insert key/value pairs in the container.
    ///
    /// It is up to the implementation to decide whether redundant keys can be
    /// inserted or not.
    ///
    /// If the container cannot store all the values, some values are returned.
    /// It is up to the implementation to decide which elements get evicted or
    /// if all the values are attempted to be inserted once the container is
    /// full.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    ///
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    /// For instance, when flushing a large number of elements out of a file,
    /// the file containing the elements can be moved into a temporary file,
    /// and an iterator over that file elements could be returned.
    fn flush(&mut self) -> Self::FlushIterator;
}

impl<K, V, C> BuildingBlock<K, V> for &'_ mut C
where
    C: BuildingBlock<K, V>,
{
    type FlushIterator = C::FlushIterator;
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
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        (**self).take_multiple(keys)
    }
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        (**self).pop(size)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        (**self).push(values)
    }
    fn flush(&mut self) -> Self::FlushIterator {
        (**self).flush()
    }
}

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
    /// Get the maximum number of elements fitting in the container.
    /// The actual number may be smaller depending on the implementation.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a matchig key.
    fn contains(&self, key: &K) -> bool;

    /// Take the matching key/value pair out of the container.
    fn take(&mut self, key: &K) -> Option<(K, V)>;

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

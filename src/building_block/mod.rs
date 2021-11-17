/// Mark a [`building blocks`](trait.BuildingBlock.html) as thread safe.
/// When this trait is implemented, the implementer guarantees that the
/// container can be used safely concurrently in between its clones
/// obtained with the method
/// [`Concurrent::clone()`](trait.Concurrent.html#tymethod.clone).
/// Clones of this container are shallow copies referring to the same
/// building block.
pub trait Concurrent<'a, K: 'a, V: 'a>:
    crate::BuildingBlock<'a, K, V> + Send + Sync
{
    /// Create a shallow copy of the container pointing to the same
    /// container that can be later used concurrently.
    fn clone(&self) -> Self;
}

/// [`BuildingBlock`](../trait.BuildingBlock.html)s connecting two
/// [`BuildingBlock`](../trait.BuildingBlock.html)s
pub mod connector;
/// [`BuildingBlock`](../trait.BuildingBlock.html)s storing key/value pairs.
pub mod container;
/// [`BuildingBlock`](../trait.BuildingBlock.html) splitting the key/value
/// pairs traffic into multiple
/// [`BuildingBlock`](../trait.BuildingBlock.html)s.
pub mod multiplexer;
/// [`BuildingBlock`](../trait.BuildingBlock.html) wrapping another
/// [`BuildingBlock`](../trait.BuildingBlock.html) to, for instance,
/// augment information in key value pairs with metadata or record
/// [`BuildingBlock`](../trait.BuildingBlock.html) usage statistics.
pub mod wrapper;

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html) to make it thread safe.
///
/// Mark a [`building block`](trait.BuildingBlock.html) as thread safe.
/// When this trait is implemented, the implementer guarantees that the
/// container can be used safely concurrently in between its clones
/// obtained with the method
/// [`Concurrent::clone()`](trait.Concurrent.html#tymethod.clone).
pub trait Concurrent: Send + Sync {
    /// Create a shallow copy of the container pointing to the same
    /// container that can be later used concurrently.
    fn clone(&self) -> Self;
}

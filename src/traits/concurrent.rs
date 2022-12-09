/// A clonable concurrent `BuildingBlock` with thread safe clones.
///
/// This trait allows to clone a [`BuildingBlock`](trait.BuildingBlock.html)
/// into a shadow copies of itself. Each clone obtained with the method
/// [`clone()`](trait.Concurrent.html#tymethod.clone) represent handles to the
/// same container and can be used safely in a threaded environment.
pub trait Concurrent: Send + Sync {
    /// Create a shallow copy of the container pointing to the same
    /// container that can be later used concurrently.
    fn clone(&self) -> Self;
}

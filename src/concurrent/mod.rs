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

mod sequential;
pub use crate::concurrent::sequential::{LockedItem, Sequential};

mod associative;
pub use crate::concurrent::associative::Associative;

#[cfg(test)]
/// Public test module available only for testing concurrent implementation.
pub mod tests;

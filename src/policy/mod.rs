/// Ordering policy wrapper for cache values.
///
/// A reference is an ordering wrapper for values that live in a
/// [building block](../trait.BuildingBlock.html).
/// This trait implements an ordering of victims in containers
/// and therefore an eviction policy for containers implementing the
/// [`Ordered trait`].
pub trait Reference<V>: Ord {
    fn unwrap(self) -> V;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
}

/// Facility to wrap cache values into a [`Reference`] cell.
pub trait ReferenceFactory<V> {
    type Item: Reference<V>;

    /// Wrap a value into a reference.
    fn wrap(&mut self, v: V) -> Self::Item;
}

/// Containers that can be used with a `Policy`.
///
/// This is a marker trait for [`BuildingBlock`](trait.BuildingBlock.html).
/// When this trait is implemented, the building blocks will try to
/// [pop](trait.BuildingBlock.html#tymethod.pop) values in descending
/// order. More importantly, is signals that it is safe to use the container
/// with [policies](index.html).
pub trait Ordered<V: std::cmp::Ord> {}

mod building_block;
mod concurrent;
pub(crate) mod get;
mod lrfu;
pub use lrfu::Lrfu;
mod lru;
pub use lru::Lru;
mod fifo;
pub use fifo::Fifo;
#[cfg(test)]
mod default;
#[cfg(test)]
pub use default::{Default, DefaultCell};
pub(crate) mod builder;
#[cfg(feature = "config")]
pub(crate) mod config;
#[allow(clippy::module_inception)]
pub(crate) mod policy;
pub mod timestamp;

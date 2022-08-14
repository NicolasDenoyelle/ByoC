/// Value ordering implementation.
///
/// A reference is a value wrapper that lives in
/// [building block](../trait.BuildingBlock.html).
/// This trait implements an ordering of victims in containers
/// to therefore an eviction policy. It also implements access
/// to the value it wraps.
pub trait Reference<V>: Ord {
    fn unwrap(self) -> V;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
}

/// Facility to wrap a value into a cache reference.
pub trait ReferenceFactory<V, R>
where
    R: Reference<V>,
{
    /// Wrap a value into a reference.
    fn wrap(&mut self, v: V) -> R;
}

mod lrfu;
pub use lrfu::LRFU;
mod lru;
pub use lru::LRU;
mod fifo;
pub use fifo::FIFO;
#[cfg(test)]
mod default;
#[cfg(test)]
pub use default::{Default, DefaultCell};
pub(crate) mod builder;
pub(crate) mod policy;
/// Fixed point in time used with some cache policy.
pub mod timestamp;

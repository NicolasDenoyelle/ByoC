/// Value ordering implementation.
///
/// A reference is a value wrapper that lives in
/// [containers](../container/index.html).
/// This trait implements an ordering of victims in containers
/// to therefore an eviction policy. It also implements access
/// to the value it wraps.
pub trait Reference<V>: Ord {
    fn unwrap(self) -> V;
    fn get<'a>(&'a self) -> &'a V;
    fn get_mut<'a>(&'a mut self) -> &'a mut V;
}

/// Facility to wrap a value into a cache reference.
pub trait ReferenceFactory<V, R>
where
    R: Reference<V>,
{
    /// Wrap a value into a reference.
    fn wrap(&mut self, v: V) -> R;
}

/// Trait of containers [popping](../trait.BuildingBlock.html#tymethod.pop)
/// highest values first.
pub trait Ordered<V: std::cmp::Ord> {}

mod policy;
pub use crate::policy::policy::Policy;
mod lrfu;
pub use crate::policy::lrfu::LRFU;
mod lru;
pub use crate::policy::lru::LRU;
mod fifo;
pub use crate::policy::fifo::FIFO;
#[cfg(test)]
mod default;
/// Fixed point in time used with some cache [policies](../policy/struct.LRFU.html).
pub mod timestamp;
#[cfg(test)]
pub use crate::policy::default::{Default, DefaultCell};
#[cfg(test)]
/// Public test module available only for testing containers implementation.
pub mod tests;

//! Eviction policies and related traits and modules.
//!
//! Containers eviction is implemented by the
//! [`building blocks`](../trait.BuildingBlock.html#implementors)
//! themselves when calling the
//! [`pop()`](../trait.BuildingBlock.html#tymethod.pop) method.
//! The when sthe container implements the [`Ordered`] trait, the pop
//! method will try to take out the elements with the highest value.
//!
//! The [`Policy`](../struct.Policy.html) container is wrapper around
//! such a container (although the container does not need to carry the
//! [`Ordered`] trait bound) that will wrap values into a [`Reference`]
//! cell ordering values in the container with a specific policy.
//!
//! This is a generic, but potentially inefficient, way to customize the
//! eviction policy on a wide range of containers.
//!
//! [`Lrfu`] and [`Lru`] policies will change the order of elements
//! in the container when they are accessed from within the container
//! using [`Get`](../trait.Get.html) and [`GetMut`](../trait.GetMut.html)
//! traits. This is potentially dangerous! Indeed, if the container relies
//! on the order of its elements (for instance it uses a
//! [`std::collections::BTreeSet`]), then
//! accessing elements inside the container will make things dicey.
//! If the container does not
//! implement the [`Ordered`] trait bound, it is probably a bad idea to use
//! on of these policies.
//!
//! ### Examples
//!
//! ```
//! use byoc::BuildingBlock;
//! use byoc::{Array, Policy};
//! use byoc::policy::Fifo;
//!
//! let mut c = Policy::new(Array::new(3), Fifo::new());
//! c.push(vec![("item1",()), ("item2",()), ("item0",())]);
//! assert_eq!(c.pop(1).pop().unwrap().0, "item1");
//! assert_eq!(c.pop(1).pop().unwrap().0, "item2");
//! assert_eq!(c.pop(1).pop().unwrap().0, "item0");
//! ```

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
pub trait ReferenceFactory<V, R>
where
    R: Reference<V>,
{
    /// Wrap a value into a reference.
    fn wrap(&mut self, v: V) -> R;
}

/// Containers that can be used with a [`Policy`](../struct.Policy.html)
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

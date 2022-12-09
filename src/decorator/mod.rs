//! The [`Decorator`](../struct.Decorator.html) container is wrapper around
//! such a container that will wrap values into
//! a [`Decoration`](trait.Decoration.html)
//! cell.
//!
//! [`Lrfu`](struct.Lrfu.html) and [`Lru`](struct.Lru.html) policies will
//! change the order of elements
//! in the container when they are accessed from within the container
//! using [`Get`](../trait.Get.html) and [`GetMut`](../trait.GetMut.html)
//! traits. This is potentially dangerous! Indeed, if the container relies
//! on the order of its elements (for instance it uses a
//! [`std::collections::BTreeSet`]), then
//! accessing elements inside the container will make things dicey.
//!
//! ### Examples
//!
//! ```
//! use byoc::BuildingBlock;
//! use byoc::{Array, Decorator};
//! use byoc::decorator::Fifo;
//!
//! let mut c = Decorator::new(Array::new(3), Fifo::new());
//! assert_eq!(c.push(vec![("item1",1u16), ("item2",2u16), ("item0",0u16)])
//!             .len(), 0);
//! assert_eq!(c.pop(1).pop().unwrap().0, "item1");
//! assert_eq!(c.pop(1).pop().unwrap().0, "item2");
//! assert_eq!(c.pop(1).pop().unwrap().0, "item0");
//! ```

/// Decorated wrapper for cache values.
///
/// A [`Decoration`] is a wrapper for values that live in a
/// [building block](../trait.BuildingBlock.html).
pub trait Decoration<V>: Ord {
    fn unwrap(self) -> V;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
}

/// Facility to wrap cache values into a [`Decoration`] cell.
pub trait DecorationFactory<V> {
    type Item: Decoration<V>;

    /// Wrap a value into a reference.
    fn wrap(&mut self, v: V) -> Self::Item;
}

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
pub(crate) mod decorator;

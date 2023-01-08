/// `BuildingBlock` values wrapper.
///
/// A [`Decoration`] is a wrapper for values that live in a
/// [building block](../trait.BuildingBlock.html). Values are wrapped using
/// a [`DecorationFactory`] when they are inserted inside a
/// [`Decorator`](../../struct.Decorator.html) container. They are unwrapped
/// using this [`Decoration`] capability and accessed using this trait method
/// when [`Get`](../../trait.Get.html) and [`GetMut`](../../trait.GetMut.html)
/// traits methods are invoked.
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

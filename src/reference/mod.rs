use std::cmp::Ord;
use std::ops::{Deref, DerefMut};

/// Definition of [`Reference`](trait.Reference.html) trait.
///
/// **References shall not implement an order based on their
/// inner pointer value.** Cache elements can be accessed and modified while in the cache.
/// If `Ord` is implemented on references value then `Ord` inside the
/// [`Container`](../container/trait.Container.html) can be broke and lead to
/// buggy eviction policy.
///
/// ### Generics:
///
/// * `V`: The type of the value held in the
/// [`Reference`](trait.Reference.html).
///
/// ### Traits:
///
/// * `Ord`: The cache eviction policy. Maximum element is the next cache victim.
/// * `Deref<Target = V>`: Read-only access to the value held in the cache Reference.
/// * `DerefMut<Target = V>`: Write access to the value held in the cache Reference.
pub trait Reference<V>: Ord + Deref<Target = V> + DerefMut<Target = V> {
    /// Construct a reference which is a copy of an existing reference,
    /// with a new value.
    fn from_ref(value: V, other: &Self) -> Self;

    /// Consume the cache reference and get ownership its inner value.
    fn unwrap(self) -> V;

    /// Function to update reference state when it is looked up in the cache.
    /// Returns self to allow chaining calls.
    fn touch(&mut self) -> &mut Self {
        self
    }

    /// Replace the value inside a reference with another value.
    fn replace(&mut self, value: V) -> V {
        std::mem::replace(self.deref_mut(), value)
    }
}

/// References that can be created with no other argument but the value they embed.
pub trait FromValue<V>: Reference<V> {
    /// Construct a cache reference.
    ///
    /// Cache reference takes ownership of value.
    /// Cache references [`Container`](../container/trait.Container.html) may need
    /// to create new references regardless of the
    /// [`Reference`](trait.Reference.html) implementation
    /// details. Therefore, generic constructor is needed for this trait.
    ///
    /// ### Arguments
    ///
    /// * `value`: The value held in the cache reference.
    fn from_value(value: V) -> Self;
}

mod priority;
pub use crate::reference::priority::Priority;
mod lrfu;
pub use crate::reference::lrfu::LRFU;
mod lru;
pub use crate::reference::lru::LRU;
mod fifo;
pub use crate::reference::fifo::FIFO;
mod default;
pub use crate::reference::default::Default;

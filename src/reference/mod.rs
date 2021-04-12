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
pub trait Reference<V>:
    Ord + Deref<Target = V> + DerefMut<Target = V>
{
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

use crate::BuildingBlock;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Metadata attached to values inside of [`Inclusive`] container.
pub struct InclusiveCell<V> {
    value: V,
    updated: bool,
    cloned: Cell<bool>,
}

impl<V> InclusiveCell<V> {
    /// Construct an inclusive cell in its initial state.
    pub(super) fn new(value: V) -> Self {
        InclusiveCell {
            value,
            updated: false,
            cloned: Cell::new(false),
        }
    }

    /// Get the value inside the container.
    pub(super) fn unwrap(self) -> V {
        self.value
    }

    /// Whether the value has been accessed mutably.
    pub(super) fn is_updated(&self) -> bool {
        self.updated
    }

    /// Whether the value has been cloned from one container to another.
    pub(super) fn is_cloned(&self) -> bool {
        self.cloned.get()
    }
}

impl<V: Clone> Clone for InclusiveCell<V> {
    fn clone(&self) -> Self {
        self.cloned.set(true);
        InclusiveCell {
            value: self.value.clone(),
            updated: self.updated,
            cloned: Cell::new(true),
        }
    }
}

impl<V> Deref for InclusiveCell<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for InclusiveCell<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We only set the updated flag to true if the value is a clone.
        // The state (updated, false) is never used.
        self.updated = self.cloned.get();
        &mut self.value
    }
}

/// Multilevel [`BuildingBlock`](trait.BuildingBlock.html) mirroring values in
/// the first level [`BuildingBlock`](trait.BuildingBlock.html).
///
/// This building block behaves has a two level cache where the front container
/// serves as a cache to the back container. In this variant, when an element
/// from the back container is [accessed](trait.Get.html),
/// it is cloned in the front container and accessed from there.
/// This avoids writes back to the back container on
/// evictions if the evicted element was not modified from its initial clone.
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// Elements are [inserted](trait.BuildingBlock.html#method.push)
/// at the front of the container. Inserted elements are
/// wrapped in a cell containing flags to keep track if the element has been
/// updated or cloned. Evicted elements go toward the back unless they are not
/// updated clones of elements from the back container.
///
/// When [taking](trait.BuildingBlock.html#method.take) elements out of the
/// container, they are first searched in the front container and then in the
/// back container if it was not in the former. If the element is found in the
/// in the front container and it is marked as cloned, it is also searched in
/// the back container and deleted.
///
/// When [popping](trait.BuildingBlock.html#method.pop) elements out,
/// this container first tries to make room in the back container using this
/// container [`pop()`](trait.BuildingBlock.html#method.pop) method. If this is
/// not enough room compared to what is requested, only then the front container
/// will also be popped from. Note that element popped from the back container
/// are not necessarily returned if they are also present in the front
/// container. They still free the room they previously occupied.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// [`Get`](trait.Get.html) and [`GetMut`](trait.GetMut.html) traits require
/// that both the front and the back container implement these traits so that
/// elements can be accessed at the front and cloned from the back to the front
/// if they were only present in the back. The associated methods will search
/// for the element first in the front container, and then in the back container
/// if it was not found in the former.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Inclusive, Array};
///
/// // Create cache
/// let mut front = Array::new(2);
/// let mut back = Array::new(4);
/// let mut cache = Inclusive::new(front, back);
/// // [[][]]
///
/// // Populate front.
/// assert!(cache.push(vec![("first", 1), ("second", 0)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][]]
///
/// // Front side is full. Next push will move the highest values
/// // from the front to the back to make room for the new
/// // value in the front.
/// assert!(cache.push(vec![("third", 3)]).pop().is_none());
/// // [[("second", 0), ("third", 3)][("first", 1)]]
///
/// // At this point, "second" and "third" are in front and
/// // "first" is in the back.
/// // Pop operation removes elements from the back first
/// // then the front.
/// assert_eq!(cache.pop(1).pop().unwrap().0, "first");
/// // [[("second", 0), ("third", 3)][]]
///
/// // We reinsert "first". As a result "third" moves to the back
/// // because the associated value is the highest on this side of the
/// // cache.
/// assert!(cache.push(vec![("first", 1)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][("third", 3)]]
///
/// // We access third. It moves it to the front, while a victim goes
/// // to the back.
/// assert!(!cache.front().contains(&"third"));
/// let val = cache.get(&"third");
/// // [[("third", 3), ("second", 0)][("first", 1)]]
/// assert!(cache.front().contains(&"third"));
/// assert!(cache.back().contains(&"first"));
///
/// // If we make room at the front then access an element at the
/// // back, this element will be both at the front and the back.
/// assert!(cache.take(&"third").is_some());
/// let val = cache.get(&"first");
/// // [[("first", 1), ("second", 0)][("first", 1)]]
/// assert!(cache.front().contains(&"first"));
/// assert!(cache.back().contains(&"first"));
/// ```
pub struct Inclusive<'a, K, V, L, R>
where
    K: 'a + Clone,
    V: 'a + Clone,
    L: BuildingBlock<'a, K, InclusiveCell<V>>,
    R: BuildingBlock<'a, K, InclusiveCell<V>>,
{
    pub(super) front: L,
    pub(super) back: R,
    pub(super) unused: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, L, R> Inclusive<'a, K, V, L, R>
where
    K: 'a + Clone,
    V: 'a + Clone,
    L: BuildingBlock<'a, K, InclusiveCell<V>>,
    R: BuildingBlock<'a, K, InclusiveCell<V>>,
{
    /// Construct a Inclusive Cache.
    pub fn new(front: L, back: R) -> Self {
        Inclusive {
            front,
            back,
            unused: PhantomData,
        }
    }

    /// Get shared access to the front container of the cache.
    pub fn front(&self) -> &L {
        &self.front
    }

    /// Get shared access to the back container of the cache.
    pub fn back(&self) -> &R {
        &self.back
    }
}

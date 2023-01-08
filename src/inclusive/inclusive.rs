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

/// Two stages `BuildingBlock` where elements in the first are copies of
/// elements in the second stage.
///
/// This building block behaves has a two level cache where the front container
/// serves as a cache to the back container.
/// The two-level structure of this container allows
/// to optimize lookups, read and writes when the most frequently accessed
/// element are kept in the fastest (front) container. This is because this
/// implementation tries first to find elements in the front container and will
/// not access the back container if the needed elements were found in the
/// former.
///
/// Elements in the front of an [`Inclusive`] container are also present in
/// the back container. Consequently, the front container
/// capacity must be at least as large as the back container capacity.
/// In practice, it is implemented using clones in the
/// front container and updating their counterpart in the back container when
/// they are evicted from the front container towards the back. Elements are
/// wrapped in a cell containing flags to keep track if the element has been
/// updated (in the front container) or cloned. It allows to avoid
/// unnecessary writes-back, when for instance, an element evicted from the
/// front to the back has not been updated or when an element evicted from
/// the back has not been cloned and therefore does not have an updated clone
/// to be updated from.
///
/// When accessing elements with [`Get`](trait.Get.html) and
/// [`GetMut`](trait.GetMut.html) traits, elements found in the back container
/// but not in the front container will be copied in the front to give
/// it a chance to be in the "faster" container on its next access.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Inclusive, Array};
///
/// // Create cache
/// let mut front = Array::new(2);
/// let mut back = Array::new(3);
/// let mut cache = Inclusive::new(front, back);
/// // [[][]] This represent the content of both front and back containers.
///
/// // Insertion happens both at the front and the back.
/// // The back is necessary because this is an inclusive cache.
/// // The front is because we assume that recent insertion of elements will
/// // be used soon.
/// assert!(cache.push(vec![("first", 1), ("second", 0)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][("first", 1), ("second", 0)]]
///
/// // Front side is full. Next push will move the highest values
/// // from the front to the back to make room for the new
/// // value in the front. The "highest values" is because
/// // `Array` (front) container pops highest values when it is full.
/// assert!(cache.push(vec![("third", 3)]).pop().is_none());
/// // [[("second", 0), ("third", 3)]
/// //  [("first", 1), ("second", 0), ("third", 3)]]
/// assert!(!cache.front().contains(&"first"));
/// assert!(cache.back().contains(&"first"));
///
/// // Pop operation removes elements from the back first and also from the
/// // the front if it is present there.
/// assert_eq!(cache.pop(1).pop().unwrap().0, "third");
/// // [[("second", 0)][("first", 1), ("second", 0)]]
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
    /// Construct an [`Inclusive`] Cache with two stages.
    ///
    /// The `front` stage should be the "faster" container, usually with
    /// a smaller capacity then the `back` stage. The `back` is also usually
    /// the "slower" container. When an element is accessed, it is copied to
    /// the front to make its subsequent accesses faster.
    pub fn new(front: L, back: R) -> Self {
        assert!(front.capacity() <= back.capacity());
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

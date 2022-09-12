use std::marker::PhantomData;

/// Connector between two [`BuildingBlock`](trait.BuildingBlock.html) without
/// duplicates.
///
/// This building block behaves has a two level cache where the front container
/// serves as a cache to the back container. In this variant, elements move
/// from the front to the back of the container and no copy is held in the
/// back container when that happens.
///
/// Insertions happen at the front of the container. If the front side does not
/// have enough room for all the elements to insert, a sufficient amount of
/// items inside the front container is evicted toward the back, i.e
/// ([popped](../trait.BuildingBlock.html#tymethod.pop) from the front then
/// [pushed](../trait.BuildingBlock.html#tymethod.push)) to the back container.
/// If there is more elements to insert than the capacity of the front
/// container, the front container implementation of the
/// [`push()`](../trait.BuildingBlock.html#tymethod.push) method decides which
/// one are inserted at the front and which ones are pushed at the back along
/// with eviction victims. If not all elements fit at the back, the
/// [`push()`](../trait.BuildingBlock.html#tymethod.push) method of the back
/// container decides which elements are returned and which are inserted.
///
/// Evictions try to remove elements from the back container first with
/// the aforementioned container
/// ([`pop()`](../trait.BuildingBlock.html#tymethod.pop) method.
/// If the amount of element in the back container is less than the amount to
/// pop, then the method is also called on the front container.
///
/// Elements lookup and removal based on a key happen first at the front and
/// if not all the keys were match, then the remaining unmatched keys
/// are looked up in the back container.
///
/// [`Get`](trait.Get.html) and [`GetMut`](trait.GetMut.html) traits require
/// that both the front and the back container implement these traits.
/// When their associated methods are called, elements do not directly move
/// from the front to the back and vice versa. Instead, they are searched first
/// in the front container and then in the back container and returned from
/// there if a key match was found. It is up to the user to bring accessed
/// elements to the front of the cache by using
/// [`take()`](../trait.BuildingBlock.html#tymethod.take) and
/// [`push()`](../trait.BuildingBlock.html#tymethod.push) methods instead.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, GetMut};
/// use byoc::{Exclusive, Array};
///
/// // Create cache
/// let element_size = Array::<(&str,u32)>::element_size();
/// let mut front = Array::new(2 * element_size);
/// let mut back = Array::new(4 * element_size);
/// let mut cache = Exclusive::new(front, back);
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
/// ```
///
/// [`Exclusive`] can also be built from a
/// [builder pattern](builder/trait.Build.html#method.exclusive) and a
/// [configuration](config/struct.ExclusiveConfig.html).
pub struct Exclusive<K, V, L, R> {
    pub(super) front: L,
    pub(super) back: R,
    pub(super) unused: PhantomData<(K, V)>,
}

impl<K, V, L, R> Exclusive<K, V, L, R> {
    /// Construct a Exclusive Cache.
    pub fn new(front: L, back: R) -> Self {
        Exclusive {
            front,
            back,
            unused: PhantomData,
        }
    }

    /// Get shared access to the front container of the cache.
    pub fn front(&self) -> &L {
        &self.front
    }

    /// Get exclusive access to the front container of the cache.
    pub fn front_mut(&mut self) -> &mut L {
        &mut self.front
    }

    /// Get shared access to the back container of the cache.
    pub fn back(&self) -> &R {
        &self.back
    }

    /// Get exclusive access to the back container of the cache.
    pub fn back_mut(&mut self) -> &mut R {
        &mut self.back
    }
}

impl<K, V, L, R> From<(L, R)> for Exclusive<K, V, L, R> {
    fn from(lr: (L, R)) -> Self {
        Exclusive {
            front: lr.0,
            back: lr.1,
            unused: PhantomData,
        }
    }
}

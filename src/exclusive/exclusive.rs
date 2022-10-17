use crate::BuildingBlock;
use std::marker::PhantomData;

/// Multilevel `BuildingBlock` withoutduplicates between levels.
///
/// This building block behaves has a two level cache where the front container
/// serves as a cache to the back container. In this variant, elements move
/// from the front to the back of the container and no copy is held in the
/// back container when that happens.
///
/// [`Exclusive`] can also be built from a
/// [builder pattern](builder/trait.Build.html#method.exclusive) and a
/// [configuration](config/struct.ExclusiveConfig.html).
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
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
/// If the freed size in the back container is less than the amount to
/// pop, then the method is also called on the front container to attempt to
/// free remaining size.
///
/// Elements lookup and removal based on a key happen first at the front and
/// if not all the keys were match, then the remaining unmatched keys
/// are looked up in the back container.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// [`Get`](trait.Get.html) and [`GetMut`](trait.GetMut.html) traits require
/// that only the front container implement these traits.
/// When their associated methods are called, if the element is not found in
/// the front, but is found in the back, it is moved from the front to the back.
/// Theredore, if the target element is found, it is returned from the front
/// container always.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, GetMut};
/// use byoc::{Exclusive, Array};
///
/// // Create cache
/// let mut front = Array::new(2);
/// let mut back = Array::new(4);
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
pub struct Exclusive<'a, K, V, L, R>
where
    K: 'a,
    V: 'a,
    L: BuildingBlock<'a, K, V>,
    R: BuildingBlock<'a, K, V>,
{
    pub(super) front: L,
    pub(super) back: R,
    pub(super) unused: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, L, R> Exclusive<'a, K, V, L, R>
where
    K: 'a,
    V: 'a,
    L: BuildingBlock<'a, K, V>,
    R: BuildingBlock<'a, K, V>,
{
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

    /// Move an element from the back container to the front container
    /// and return whether the element was here at all.
    pub(super) fn downgrade(&mut self, key: &K) -> bool {
        // Lookup in the back stage of the cache.
        let x = match self.back.take(key) {
            // If element is not there, there's no downgrade possible.
            None => return false,
            Some(x) => x,
        };

        // Insert element in the front stage of the cache.
        let popped = self.front.push(vec![x]);
        if popped.is_empty() {
            return true;
        }

        // If Some elements were popped we try to insert them at the back.
        let popped = self.back.push(popped);
        if popped.is_empty() {
            return true;
        }

        // If we have yet more elements popping out, we have nowhere to put them
        // and we cannot roll back, so we have to abort.
        panic!("Failure to move an element from the back container of the exclusive cache to the front. This happened because we tried to move an element from the back container to the front, then the front container popped elements to make room for the former element, and finally, the back container was not able to fit the popped elements.")
    }
}

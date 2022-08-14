use std::marker::PhantomData;

/// Connect two building blocks with key/values going forward from the
/// left container to the right container.
///
/// This building block typically implements the policy for how data is
/// moved from one level to another in a multi-level cache, where the left
/// building block is used as a cache for the right building block.
/// Elements transfer from one side to another only happen when the cache
/// is accessed in a mutable fashion, mainly when calling
/// [`push()`](../trait.BuildingBlock.html#tymethod.push) or
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop).
///
/// See `BuildingBlock` and `Get` implementations details for more
/// information on data movement implementation.
///
/// # Examples
///
/// ```
/// use byoc::{BuildingBlock, GetMut};
/// use byoc::{Multilevel, Array};
///
/// // Create cache
/// let mut left = Array::new(2);
/// let mut right = Array::new(4);
/// let mut cache = Multilevel::new(left, right);
/// // [[][]]
///
/// // Populate left side.
/// assert!(cache.push(vec![("first", 1), ("second", 0)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][]]
///
/// // Left side is full. Next push will move the highest values
/// // from the left side to the right side to make room for the new
/// // value in the left side.
/// assert!(cache.push(vec![("third", 3)]).pop().is_none());
/// // [[("second", 0), ("third", 3)][("first", 1)]]
///
/// // At this point, "second" and "third" are in left side and
/// // "first" is in the right side.
/// // Pop operation removes elements from the right side first
/// // then the left side.
/// assert_eq!(cache.pop(1).pop().unwrap().0, "first");
/// // [[("second", 0), ("third", 3)][]]
///
/// // We reinsert "first". As a result "third" moves to the right side
/// // because the associated value is the highest on this side of the
/// // cache.
/// assert!(cache.push(vec![("first", 1)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][("third", 3)]]
///
/// // Accessing a (mutable) value promotes it to the left side.
/// // If we access "third", it moves to the left side while "first",
/// // will go back to the right side because it is the highest value.
/// let mut val = unsafe { cache.get_mut(&"third") };
/// // [[("second", 0), ("third", 3)][("first", 1)]]
/// assert_eq!(cache.pop(1).pop().unwrap().0, "first")
/// // [[("second", 0), ("third", 3)][]]
/// ```
pub struct Multilevel<K, V, L, R> {
    pub(super) left: L,
    pub(super) right: R,
    pub(super) unused: PhantomData<(K, V)>,
}

impl<K, V, L, R> Multilevel<K, V, L, R> {
    /// Construct a Multilevel Cache.
    pub fn new(left: L, right: R) -> Self {
        Multilevel {
            left,
            right,
            unused: PhantomData,
        }
    }
}

impl<K, V, L, R> From<(L, R)> for Multilevel<K, V, L, R> {
    fn from(lr: (L, R)) -> Self {
        Multilevel {
            left: lr.0,
            right: lr.1,
            unused: PhantomData,
        }
    }
}

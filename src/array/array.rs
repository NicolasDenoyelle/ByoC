use std::vec::Vec;

/// In-memory container implementation as a fixed size array of key/value pairs.
///
/// [`Array`] is an unordered container built on top of a [`std::vec::Vec`].
///
/// * Insertion complexity is `$O(n)$`.
/// The whole array is walked to look for matching keys and avoid collisions.
/// * Removal complexity is `$O(n)$`.
/// The whole array is walked to look for matching keys.
/// * Eviction complexity is `$O(n*log(n))$`.
/// The whole array is sorted to remove the top `k` elements.
/// * Keys lookup complexity is `$O(n)$`.
/// * Capacity and count queries are `$O(1)$`.
///
/// Removal performance can be slightly better using the
/// `take_multiple()` method.
/// The removal complexity is `$O(k*log(k) + n*log(k))$` where `n` is the number
/// of elements in the container and `k` is the number of keys to lookup.
///
/// This container implements the [`Ordered`](../policy/trait.Ordered.html)
/// trait and can be safely used with a custom [policy](policy/index.html).
///
/// Elements within the array can be accessed with the [`Get`](trait.Get.html)
/// [`GetMut`](trait.GetMut.html) traits. These traits return a pointer to
/// an element inside the underlying [`std::vec::Vec`] and are safe to use
/// as long as the borrow rule are respected.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::Array;
///
/// // Array with 3 elements capacity.
/// let mut c = Array::new(3);
///
/// // BuildingBlock as room for 3 elements and returns an empty array.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4),
///                     ("second", 2),
///                     ("third", 3)]).pop().is_none());
///
/// // Array is full and pops extra inserted value (all values here).
/// let (key, _) = c.push(vec![("fourth", 12)]).pop().unwrap();
/// assert_eq!(key, "fourth");
///
/// // Array pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "first");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "third");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "second");
/// ```
///
/// [`Array`] can also be built from a
/// [builder pattern](builder/struct.Builder.html#method.array) and a
/// [configuration](config/struct.ArrayConfig.html).
pub struct Array<T> {
    pub(super) capacity: usize,
    pub(super) values: Vec<T>,
}

impl<T> Array<T> {
    pub fn new(n: usize) -> Self {
        Array {
            capacity: n,
            values: Vec::with_capacity(n),
        }
    }
}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array {
            capacity: self.capacity,
            values: self.values.clone(),
        }
    }
}

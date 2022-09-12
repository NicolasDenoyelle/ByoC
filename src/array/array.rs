use std::vec::Vec;

/// In-memory container implementation as a fixed size array of key/value pairs.
///
/// [`Array`] is an unordered container built on top of a [`std::vec::Vec`].
///
/// The number of elements fitting in this container is computed as the sum of
/// its [elements size](struct.Array.html#method.element_size)
/// where `std::mem::size_of::<(K,V)>()` is the size of one key/value pair
/// element. Hence, only the stack size of elements is counted and data stored
/// within a [`std::boxed::Box`] is not counted as occupying container space.
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
/// [`take_multiple()`](struct.Array.html#method.take_multiple) method.
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
/// let mut c = Array::new(3 * Array::<(&str, i32)>::element_size());
///
/// // BuildingBlock as room for 3 elements and returns an empty array.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4),
///                     ("second", 2),
///                     ("third", 3)]).pop().is_none());
///
/// // Array is full and pops extra inserted value (all values here).
/// let mut popped = c.push(vec![("fourth", 12)]);
/// assert_eq!(popped.len(), 1);
/// let (key, _) = popped.pop().unwrap();
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
    // Capacity in number of elements. Elements are Sized.
    pub(super) max_elements: usize,
    pub(super) capacity: usize,
    pub(super) values: Vec<T>,
}

impl<T> Array<T> {
    pub fn new(size: usize) -> Self {
        let n = size / Self::element_size();
        Array {
            max_elements: n,
            capacity: size,
            values: Vec::with_capacity(n),
        }
    }

    /// Return the size of one element in the container.
    pub fn element_size() -> usize {
        std::mem::size_of::<T>()
    }
}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array {
            max_elements: self.max_elements,
            capacity: self.capacity,
            values: self.values.clone(),
        }
    }
}

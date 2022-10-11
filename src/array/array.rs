use std::vec::Vec;

/// In-memory container implementation as a fixed size array of key/value pairs.
///
/// [`Array`] is an unordered container built on top of a [`std::vec::Vec`].
///
/// The number of elements fitting in this container is computed as the sum of
/// its [elements size](struct.Array.html#method.with_element_size)
/// where the default size for a key/value pair is 1.
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
/// let mut c = Array::new(3);
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
    pub(super) capacity: usize,
    pub(super) total_size: usize,
    pub(super) values: Vec<T>,
    pub(super) element_size: fn(&T) -> usize,
}

impl<T> Array<T> {
    /// Create a new [`Array`] container with `size`
    /// [`capacity`](struct.Array.html#method.capacity).
    ///
    /// The meaning of this capacity depends on the `element_size` function
    /// set with
    /// [`with_element_size()`](struct.Array.html#method.with_element_size).
    /// The default is to set every elements size to `1usize` and therefore,
    /// `size` stands for the maximum number of elements fitting in the
    /// [`Array`].
    pub fn new(size: usize) -> Self {
        Array {
            total_size: 0,
            capacity: size,
            values: Vec::new(),
            element_size: |_| 1,
        }
    }

    /// Set how [`Array`] elements size is computed.
    ///
    /// Whenever an element is inserted or removed from the [`Array`],
    /// its size is compared with the container capacity and its remaining
    /// space to decide respectively, whether the element can be inserted or
    /// how much space does it leaves in the container.
    /// This function decides how to compute each element size and therefore
    /// it also decides of the meaning of the container
    /// [`capacity`](struct.Array.html#method.capacity).
    pub fn with_element_size(
        mut self,
        element_size: fn(&T) -> usize,
    ) -> Self {
        if self.total_size > 0 {
            panic!("It is not allowed to set a non empty Array container element_size method.")
        }
        self.element_size = element_size;
        self
    }
}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array {
            capacity: self.capacity,
            total_size: self.total_size,
            values: self.values.clone(),
            element_size: self.element_size,
        }
    }
}

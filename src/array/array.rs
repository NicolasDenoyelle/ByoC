use std::vec::Vec;

/// In-memory container implementation as a fixed size array of key/value pairs.
///
/// [`Array`] is a container that stores its elements in a [`std::vec::Vec`].
///
/// Elements that go into an [`Array`] are sized with a function
/// `element_size()` set with the method
/// [`with_element_size()`](struct.Array.html#method.with_element_size) such
/// that the sum of the sizes of [`Array`] elements never exceed its set
/// capacity. The default size for a key/value pair element is `1` and therefore
/// the container capacity in this circumstance is the number of elements it can
/// contain.
///
/// Elements within an [`Array`] are stored out of order, however, when
/// [popping](struct.struct.Array.html#method.pop) out, the elements with
/// the greatest values are evicted first. The same applies when pushing more
/// elements that can fit into the container, i.e the greatest elements within
/// the container are popped before inserting the new elements.
///
/// See [`BuildingBlock` implementation](struct.Array.html#impl-BuildingBlock)
/// for more detail on how does the container operates.
///
/// Elements within the array can be accessed with the [`Get`](trait.Get.html)
/// [`GetMut`](trait.GetMut.html) traits. These traits return a pointer to
/// an element inside the underlying [`std::vec::Vec`].
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
/// // Array is full and pops a victim with the highest value within the
/// // container before inserting the new value.
/// let mut popped = c.push(vec![("fourth", 12)]);
/// assert_eq!(popped.len(), 1);
/// let (key, _) = popped.pop().unwrap();
/// assert_eq!(key, "first");
///
/// // Array pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "fourth");
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
    /// The meaning of this capacity depends on the `element_size()` function
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

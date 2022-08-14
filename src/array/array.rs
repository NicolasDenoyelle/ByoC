use std::vec::Vec;

/// BuildingBlock implementation in as an array of key/value pairs.
///
/// # Examples
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

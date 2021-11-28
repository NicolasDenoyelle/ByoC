use crate::policy::Ordered;
use crate::{BuildingBlock, Get};
use std::cmp::Eq;
use std::ops::{Deref, DerefMut};
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Vector struct
//-------------------------------------------------------------------------

/// [`BuildingBlock`](../trait.BuildingBlock.html) implementation in a
/// vector.
///
/// Vector holds values in a `Vec<(key, value)>`.   
/// See
/// [`BuildingBlock methods implementation`](struct.Vector.html#impl-BuildingBlock%3C%27a%2C%20K%2C%20V%3E)
/// for behavior on `push()` and `pop()`.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
///
/// // Vector with 3 elements capacity.
/// let mut c = Vector::new(3);
///
/// // BuildingBlock as room for 3 elements and returns an empty vector.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4), ("second", 2), ("third", 3)]).pop().is_none());
///
/// // Vector is full and pops extra inserted value (all values here).
/// let (key, _) = c.push(vec![("fourth", 12)]).pop().unwrap();
/// assert_eq!(key, "fourth");
///
/// // Vector pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "first");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "third");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "second");
/// ```
pub struct Vector<T> {
    capacity: usize,
    values: Vec<T>,
}

impl<T> Vector<T> {
    pub fn new(n: usize) -> Self {
        Vector {
            capacity: n,
            values: Vec::with_capacity(n),
        }
    }
}

//------------------------------------------------------------------------//
//  BuildingBlock implementation.                                             //
//------------------------------------------------------------------------//

impl<'a, K, V> BuildingBlock<'a, K, V> for Vector<(K, V)>
where
    K: 'a + Eq,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.values.split_off(0).into_iter())
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(k, _)| k == key)
    }

    fn count(&self) -> usize {
        return self.values.len();
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
		/// This building block implements the trait
		/// [`Ordered`](../policy/trait.Ordered.html), which means that
		/// the highest values are popped out. This is implemented by
		/// sorting the vector on values and spitting it where appropriate.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
        let i = self.values.len();
        self.values.split_off(i - std::cmp::min(i, n))
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
		/// returned.
    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = std::cmp::min(
            self.capacity - self.values.len(),
            elements.len(),
        );
        let out = elements.split_off(n);

        if n > 0 {
            self.values.append(&mut elements);
        }
        out
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.values.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i.clone())
            } else {
                None
            }
        }) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i)),
        }
    }
}

// Make this container usable with a policy.
impl<K, V: Ord> Ordered<V> for Vector<(K, V)> {}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

/// Read-only cell representing a reference to a value inside a
/// [`Vector`](struct.Vector.html) container.
pub struct VecCell<T> {
    t: *const T,
}

impl<T> Deref for VecCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref().unwrap() }
    }
}

/// Read-write cell holding a reference to a value inside a
/// [`Vector`](struct.Vector.html) container.
pub struct VecMutCell<T> {
    t: *mut T,
}

impl<T> Deref for VecMutCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref().unwrap() }
    }
}

impl<T> DerefMut for VecMutCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.t.as_mut().unwrap() }
    }
}

impl<K: Eq, V> Get<K, V, VecCell<V>, VecMutCell<V>> for Vector<(K, V)> {
    unsafe fn get(&self, key: &K) -> Option<VecCell<V>> {
        self.values.iter().find_map(move |(k, v)| {
            if k == key {
                Some(VecCell { t: v })
            } else {
                None
            }
        })
    }

    unsafe fn get_mut(&mut self, key: &K) -> Option<VecMutCell<V>> {
        self.values.iter_mut().find_map(move |(k, v)| {
            if k == key {
                Some(VecMutCell { t: v })
            } else {
                None
            }
        })
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Vector;
    use crate::policy::tests::test_ordered;
    use crate::tests::{test_building_block, test_get};

    #[test]
    fn building_block() {
        test_building_block(Vector::new(0));
        test_building_block(Vector::new(10));
        test_building_block(Vector::new(100));
    }

    #[test]
    fn ordered() {
        test_ordered(Vector::new(0));
        test_ordered(Vector::new(10));
        test_ordered(Vector::new(100));
    }

    #[test]
    fn get() {
        test_get(Vector::new(0));
        test_get(Vector::new(10));
        test_get(Vector::new(100));
    }
}

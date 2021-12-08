use crate::{BuildingBlock, Get, GetMut, Ordered, Prefetch};
use std::cmp::Eq;
use std::ops::{Deref, DerefMut};
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Array struct
//-------------------------------------------------------------------------

/// [`BuildingBlock`](../trait.BuildingBlock.html) implementation in a
/// array.
///
/// Array holds values in a `Vec<(key, value)>`.   
/// See
/// [`BuildingBlock methods implementation`](struct.Array.html#impl-BuildingBlock%3C%27a%2C%20K%2C%20V%3E)
/// for behavior on `push()` and `pop()`.
///
/// ## Safety
///
/// See
/// [`Get methods implementation`](struct.Array.html#impl-Get%3CK%2C%20V%2C%20ArrayCell%3CV%3E%2C%20ArrayMutCell%3CV%3E%3E).
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Array;
///
/// // Array with 3 elements capacity.
/// let mut c = Array::new(3);
///
/// // BuildingBlock as room for 3 elements and returns an empty array.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4), ("second", 2), ("third", 3)]).pop().is_none());
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
    capacity: usize,
    values: Vec<T>,
}

impl<T> Array<T> {
    pub fn new(n: usize) -> Self {
        Array {
            capacity: n,
            values: Vec::with_capacity(n),
        }
    }
}

impl<K: Eq, V> Prefetch<K> for Array<(K, V)> {
    fn prefetch(&mut self, mut keys: Vec<K>) {
        if (keys.len()) == 0 {
            return;
        }
        let mut values = Vec::with_capacity(self.capacity);
        let mut is_prefetch = Vec::new();
        let mut others = Vec::new();

        std::mem::swap(&mut self.values, &mut values);

        for (k, v) in values.into_iter() {
            match keys.iter().enumerate().find_map(|(i, _k)| {
                match _k == &k {
                    true => Some(i),
                    false => None,
                }
            }) {
                Some(i) => {
                    keys.swap_remove(i);
                    is_prefetch.push((k, v));
                }
                None => others.push((k, v)),
            }
        }

        self.values.append(&mut is_prefetch);
        self.values.append(&mut others);
    }
}

//------------------------------------------------------------------------//
// BuildingBlock implementation
//------------------------------------------------------------------------//

impl<'a, K, V> BuildingBlock<'a, K, V> for Array<(K, V)>
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
    /// the returned array contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../policy/trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// sorting the array on values and spitting it where appropriate.
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
impl<K, V: Ord> Ordered<V> for Array<(K, V)> {}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

/// Read-only cell representing a reference to a value inside a
/// [`Array`](struct.Array.html) container.
pub struct ArrayCell<T> {
    t: *const T,
}

impl<T> Deref for ArrayCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref().unwrap() }
    }
}

/// Read-write cell holding a reference to a value inside a
/// [`Array`](struct.Array.html) container.
pub struct ArrayMutCell<T> {
    t: *mut T,
}

impl<T> Deref for ArrayMutCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref().unwrap() }
    }
}

impl<T> DerefMut for ArrayMutCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.t.as_mut().unwrap() }
    }
}

impl<K: Eq, V> Get<K, V, ArrayCell<V>> for Array<(K, V)> {
    /// Get value inside a `Array`. The value is wrapped inside a
    /// [`ArrayCell`](struct.ArrayCell.html). The `ArrayCell` can
    /// further be dereferenced into a value reference.
    ///
    /// ## Safety:
    ///
    /// Using the return value inside the `ArrayCell` is unsafe and can
    /// lead to undefined behavior. The user of this method must ensure that
    /// the Array container is not modified until the `ArrayCell` is
    /// droped. Otherwise, the content of the `ArrayCell` might be
    /// corrupted.
    ///
    /// ## Example:
    ///
    /// ```
    /// use cache::{BuildingBlock, Get};
    /// use cache::container::Array;
    ///
    /// // Make a array and populate it.
    /// let mut v = Array::new(1);
    /// v.push(vec![(1,1)]);
    ///
    /// // Get the value inside the array.
    /// let val = unsafe { v.get(&1).unwrap() };
    ///
    /// // Replace with another value.
    /// v.flush();
    /// v.push(vec![(2,2)]);
    ///
    /// // Val is corrupted and should not be accessible.
    /// assert!(*val != 1);
    /// ```
    unsafe fn get(&self, key: &K) -> Option<ArrayCell<V>> {
        self.values.iter().find_map(move |(k, v)| {
            if k == key {
                Some(ArrayCell { t: v })
            } else {
                None
            }
        })
    }
}

impl<K: Eq, V> GetMut<K, V, ArrayMutCell<V>> for Array<(K, V)> {
    /// Get value inside a `Array`. The value is wrapped inside a
    /// [`ArrayMutCell`](struct.ArrayMutCell.html). The `ArrayMutCell`
    /// can further be dereferenced into a value reference.
    ///
    /// ## Safety:
    ///
    /// Using the return value inside the `ArrayMutCell` is unsafe and can
    /// lead to undefined behavior. The user of this method must ensure that
    /// the Array container is not modified until the `ArrayMutCell` is
    /// droped. Otherwise, the content of the `ArrayMutCell` might be
    /// corrupted.
    ///
    /// ## Example:
    ///
    /// ```
    /// use cache::{BuildingBlock, GetMut};
    /// use cache::container::Array;
    ///
    /// // Make a array and populate it.
    /// let mut v = Array::new(1);
    /// v.push(vec![(1,1)]);
    ///
    /// // Get the value inside the array.
    /// let mut val = unsafe { v.get_mut(&1).unwrap() };
    ///
    /// // Replace with another value.
    /// v.flush();
    /// v.push(vec![(2,2)]);
    ///
    /// // Val is corrupted and should not be accessible.
    /// assert!(*val != 1);
    /// ```
    unsafe fn get_mut(&mut self, key: &K) -> Option<ArrayMutCell<V>> {
        self.values.iter_mut().find_map(move |(k, v)| {
            if k == key {
                Some(ArrayMutCell { t: v })
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
    use super::Array;
    use crate::policy::tests::test_ordered;
    use crate::tests::{test_building_block, test_get, test_get_mut};

    #[test]
    fn building_block() {
        test_building_block(Array::new(0));
        test_building_block(Array::new(10));
        test_building_block(Array::new(100));
    }

    #[test]
    fn ordered() {
        test_ordered(Array::new(0));
        test_ordered(Array::new(10));
        test_ordered(Array::new(100));
    }

    #[test]
    fn get() {
        test_get(Array::new(0));
        test_get(Array::new(10));
        test_get(Array::new(100));
        test_get_mut(Array::new(0));
        test_get_mut(Array::new(10));
        test_get_mut(Array::new(100));
    }
}

use super::Array;
use crate::{Get, GetMut};
use std::ops::{Deref, DerefMut};

/// Read-only cell representing a reference to a value inside a
/// [`Array`](struct.Array.html) container.
pub struct ArrayCell<T> {
    t: *const T,
}

impl<T> Deref for ArrayCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // Because this element is created with the unsafe get() method,
        // by contract, the user is responsible for ensuring that
        // this array element does outlive its array container,
        // and that the array is not modified/reallocated while this element
        // is being read.
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
        // SAFETY:
        // Because this element is created with the unsafe get_mut() method,
        // by contract, the user is responsible for ensuring that
        // this array element does outlive its array container,
        // and that the array is not modified/reallocated while this element
        // is being read.
        unsafe { self.t.as_ref().unwrap() }
    }
}

impl<T> DerefMut for ArrayMutCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // See Deref trait method.
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
    /// dropped. Otherwise, the content of the `ArrayCell` might be
    /// corrupted or even point to a non allocated area.
    ///
    /// ## Example:
    ///
    /// ```
    /// use byoc::{BuildingBlock, Get};
    /// use byoc::Array;
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
    /// dropped. Otherwise, the content of the `ArrayMutCell` might be
    /// corrupted or even point to a non allocated area.
    ///
    /// ## Example:
    ///
    /// ```
    /// use byoc::{BuildingBlock, GetMut};
    /// use byoc::Array;
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

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::tests::{test_get, test_get_mut};

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

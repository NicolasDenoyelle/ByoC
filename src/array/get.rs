use super::Array;
use crate::utils::get::LifeTimeGuard;
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

impl<K: Eq, V> Get<K, V> for Array<(K, V)> {
    type Target = ArrayCell<V>;

    fn get(&self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.values.iter().find_map(move |(k, v)| {
            if k == key {
                Some(LifeTimeGuard::new(ArrayCell { t: v }))
            } else {
                None
            }
        })
    }
}

impl<K: Eq, V> GetMut<K, V> for Array<(K, V)> {
    type Target = ArrayMutCell<V>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.values.iter_mut().find_map(move |(k, v)| {
            if k == key {
                Some(LifeTimeGuard::new(ArrayMutCell { t: v }))
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

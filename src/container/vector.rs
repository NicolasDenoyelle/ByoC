use crate::policy::Ordered;
use crate::{BuildingBlock, Get};
use std::cmp::Eq;
use std::ops::{Deref, DerefMut};
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Vector struct
//-------------------------------------------------------------------------

/// Unordered [`BuildingBlock`](../trait.BuildingBlock.html).
///
/// Vector holds values in a `Vec<(index, value)>`.
/// It is an unordered container.
/// Any operation on vector (
/// [`push()`](../trait.BuildingBlock.html#tymethod.push),
/// [`take()`](../trait.BuildingBlock.html#tymethod.take),
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop)
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // BuildingBlock as room for first element and returns None.
/// assert!(c.push(vec![("first", 4)]).pop().is_none());
///
/// // BuildingBlock is full and pops inserted value.
/// let (key, value) = c.push(vec![("second", 12)]).pop().unwrap();
/// assert!(key == "second");
/// assert!(value == 12);
/// ```
pub struct Vector<T> {
    capacity: usize,
    values: Vec<T>,
}

impl<T> Vector<T> {
    pub fn new(n: usize) -> Self {
        Vector {
            capacity: n,
            values: Vec::with_capacity(n + 1),
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

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
        let i = self.values.len();
        self.values.split_off(i - std::cmp::min(i, n))
    }

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

pub struct VecCell<T> {
    t: *const T,
}

impl<T> Deref for VecCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref().unwrap() }
    }
}

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
    fn get<'a>(&'a self, key: &K) -> Option<VecCell<V>> {
        self.values.iter().find_map(move |(k, v)| {
            if k == key {
                Some(VecCell { t: v })
            } else {
                None
            }
        })
    }

    fn get_mut<'a>(&'a mut self, key: &K) -> Option<VecMutCell<V>> {
        self.values.iter_mut().find_map(move |(k, v)| {
            if k == key {
                Some(VecMutCell { t: v })
            } else {
                None
            }
        })
    }
}

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

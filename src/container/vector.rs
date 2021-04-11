use crate::container::{Container, Get};
use crate::marker::Packed;
use std::cmp::Eq;
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Vector struct
//-------------------------------------------------------------------------

/// Unordered [`container`](trait.Container.html).
///
/// Vector holds values in a `Vec<(index, value)>`.
/// It is an unordered container.
/// Any operation on vector (`push()`, `pop()`, `get()`, `take()`) is O(n).
/// `push()`, `get()`, `get_mut()`, `take()` require to find a matching key
/// in the container (O(n)).
/// `pop()` requires to find a victim in the container (O(n)).
///
/// ## Generics
///
/// * `K`: The type of key to use for container lookups.
/// * `V`: Value type stored.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Vector};
/// use cache::reference::{Reference, Default};
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", Default::new(4)).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", Default::new(12)).unwrap();
///
/// // The victim is the second reference because it has a greater value.
/// assert!(key == "second");
/// assert!(*value == 12);
/// ```
pub struct Vector<K, V>
where
    K: Eq,
    V: Ord,
{
    capacity: usize,
    values: Vec<(K, V)>,
}

impl<K, V> Vector<K, V>
where
    K: Eq,
    V: Ord,
{
    pub fn new(n: usize) -> Self {
        Vector {
            capacity: n,
            values: Vec::with_capacity(n + 1),
        }
    }
}

//------------------------------------------------------------------------//
//  Container implementation.                                             //
//------------------------------------------------------------------------//

impl<K, V> Container<K, V> for Vector<K, V>
where
    K: Eq,
    V: Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        self.values.drain(..).collect()
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(k, _)| k == key)
    }

    fn count(&self) -> usize {
        return self.values.len();
    }

    fn clear(&mut self) {
        self.values.clear()
    }

    fn pop(&mut self) -> Option<(K, V)> {
        if self.count() == 0 {
            return None;
        }
        let mut v = 0;
        for i in 1..self.count() {
            if self.values[i].1 > self.values[v].1 {
                v = i
            }
        }
        let (k, r) = self.values.swap_remove(v);
        Some((k, r))
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        match self.values.iter().position(|(k, _)| k == &key) {
            None => {
                self.values.push((key, reference));
                if self.values.len() > self.capacity {
                    self.pop()
                } else {
                    None
                }
            }
            Some(i) => {
                self.values.push((key, reference));
                let (k, r) = self.values.swap_remove(i);
                Some((k, r))
            }
        }
    }

    fn take(&mut self, key: &K) -> Option<V> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i).1),
        }
    }
}

impl<'a, K, V> Get<'a, K, V> for Vector<K, V>
where
    K: Eq,
    V: 'a + Ord,
{
    type Item = &'a mut V;
    fn get(&'a mut self, key: &K) -> Option<&'a mut V> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(&mut self.values[i].1),
        }
    }
}

impl<K, V> Packed<K, V> for Vector<K, V>
where
    K: Eq,
    V: Ord,
{
}

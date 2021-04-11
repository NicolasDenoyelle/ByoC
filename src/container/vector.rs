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
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", 4).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", 12).unwrap();
///
/// // The victim is the second reference because it has a greater value.
/// assert!(key == "second");
/// assert!(*value == 12);
/// ```
pub struct Vector<V> {
    capacity: usize,
    values: Vec<V>,
}

impl<V> Vector<V> {
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

impl<K, V> Container<K, V> for Vector<(V, K)>
where
    K: Eq,
    V: Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        self.values.drain(..).map(|(v, k)| (k, v)).collect()
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(_, k)| k == key)
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
            if self.values[i].0 > self.values[v].0 {
                v = i
            }
        }
        let (v, k) = self.values.swap_remove(v);
        Some((k, v))
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        let mut victim = 0;

        for (i, (v, k)) in self.values.iter().enumerate() {
            if k == &key {
                self.values.push((reference, key));
                let (v, k) = self.values.swap_remove(victim);
                return Some((k, v));
            } else if v > &self.values[victim].0 {
                victim = i;
            }
        }

        self.values.push((reference, key));
        if self.values.len() < self.capacity {
            return None;
        } else {
            let (v, k) = self.values.swap_remove(victim);
            return Some((k, v));
        }
    }

    fn take(&mut self, key: &K) -> Option<V> {
        match self.values.iter().position(|(_, k)| k == key) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i).0),
        }
    }
}

impl<'a, K, V> Get<'a, K, V> for Vector<(V, K)>
where
    K: Eq,
    V: 'a + Ord,
{
    type Item = &'a mut V;
    fn get(&'a mut self, key: &K) -> Option<&'a mut V> {
        match self.values.iter().position(|(_, k)| k == key) {
            None => None,
            Some(i) => Some(&mut self.values[i].0),
        }
    }
}

impl<K, V> Packed<K, V> for Vector<(V, K)>
where
    K: Eq,
    V: Ord,
{
}

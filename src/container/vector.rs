use crate::container::{Container, Get, Packed};
use crate::reference::Reference;
use std::cmp::Eq;
use std::vec::Vec;

//----------------------------------------------------------------------------//
//  Vector struct                                                             //
//----------------------------------------------------------------------------//

/// [`Container`](trait.Container.html) with unordered
/// [references](../../reference/trait.Reference.html) and keys.
///
/// Vector holds references in a `Vec<(index, value)>`.
/// It is an unordered container.
/// Any operation on vector (`push()`, `pop()`, `get()`, `take()`) is O(n).
/// `push()`, `get()`, `get_mut()`, `take()` require to find a matching key
/// in the container (O(n)).
/// `pop()` requires to find a victim in the container (O(n)).
///
/// ## Generics
///
/// * `K`: The type of key to use. Keys must implement `Ord`
/// trait to be searched in the container.
/// * `V`: Value type stored in [cache reference](../../reference/trait.Reference.html).
/// * `R`: An orderable cache reference.
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
pub struct Vector<K, V, R>
where
    K: Eq,
    R: Reference<V>,
{
    wrap_ref: Box<dyn Fn(V) -> R>,
    capacity: usize,
    values: Vec<(K, R)>,
}

impl<K, V, R> Vector<K, V, R>
where
    K: Eq,
    R: Reference<V>,
{
    pub fn new(n: usize, wrap_ref: Box<dyn Fn(V) -> R>) -> Self {
        Vector {
            wrap_ref: wrap_ref,
            capacity: n,
            values: Vec::with_capacity(n + 1),
        }
    }
}

//----------------------------------------------------------------------------//
//  Container implementation.                                                 //
//----------------------------------------------------------------------------//

impl<K, V, R> Container<K, V> for Vector<K, V, R>
where
    K: Eq,
    R: Reference<V>,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        self.values
            .drain(..)
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
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
        Some((k, r.unwrap()))
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        match self.values.iter().position(|(k, _)| k == &key) {
            None => {
                self.values.push((key, (self.wrap_ref)(reference)));
                if self.values.len() > self.capacity {
                    self.pop()
                } else {
                    None
                }
            }
            Some(i) => {
                self.values.push((key, (self.wrap_ref)(reference)));
                let (k, r) = self.values.swap_remove(i);
                Some((k, r.unwrap()))
            }
        }
    }

    fn take(&mut self, key: &K) -> Option<V> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i).1.unwrap()),
        }
    }
}

impl<K, V, R> Get<K, V> for Vector<K, V, R>
where
    K: Eq,
    R: Reference<V>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(self.values[i].1.deref()),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(self.values[i].1.deref_mut()),
        }
    }
}

impl<K, V, R> Packed<K, V> for Vector<K, V, R>
where
    K: Eq,
    R: Reference<V>,
{
}

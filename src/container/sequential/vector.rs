use crate::container::{Container, Insert, Iter, IterMut, Packed, Sequential};
use crate::reference::{FromValue, Reference};
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
/// use cache::container::Container;
/// use cache::container::sequential::Vector;
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

//----------------------------------------------------------------------------//
//  Container implementation.                                                 //
//----------------------------------------------------------------------------//

impl<K: Eq, V: Ord> Container<K, V> for Vector<K, V> {
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
        Some(self.values.swap_remove(v))
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
                Some(self.values.swap_remove(i))
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

impl<K, V, R> Sequential<K, V, R> for Vector<K, R>
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
            Some(i) => {
                self.values[i].1.touch();
                Some(self.values[i].1.deref_mut())
            }
        }
    }
}

impl<K: Eq, V: Ord> Packed<K, V> for Vector<K, V> {}

impl<K: Eq, V, R: Reference<V> + FromValue<V>> Insert<K, V, R>
    for Vector<K, R>
{
}

//----------------------------------------------------------------------------//
//  Vector iterator.                                                          //
//----------------------------------------------------------------------------//

impl<'a, K, V, R> Iter<'a, K, V, R> for Vector<K, R>
where
    K: 'a + Eq,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = std::iter::Map<
        std::slice::IterMut<'a, (K, R)>,
        fn(&'a mut (K, R)) -> (&'a K, &'a V),
    >;
    fn iter(&'a mut self) -> Self::Iterator {
        self.values.iter_mut().map(|(k, r)| {
            r.touch();
            (k, r.deref())
        })
    }
}

impl<'a, K, V, R> IterMut<'a, K, V, R> for Vector<K, R>
where
    K: 'a + Eq,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = std::iter::Map<
        std::slice::IterMut<'a, (K, R)>,
        fn(&'a mut (K, R)) -> (&'a K, &'a mut V),
    >;
    fn iter_mut(&'a mut self) -> Self::Iterator {
        self.values.iter_mut().map(|(k, r)| {
            r.touch();
            (k, r.deref_mut())
        })
    }
}

use crate::container::{Container, Insert, Iter, IterMut, Sequential};
use crate::reference::{FromValue, Reference};
use std::marker::PhantomData;
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
pub struct Vector<K, V, R>
where
    K: Ord,
    R: Reference<V>,
{
    capacity: usize,
    values: Vec<(K, R)>,
    unused: PhantomData<V>,
}

impl<K, V, R> Vector<K, V, R>
where
    K: Ord,
    R: Reference<V>,
{
    pub fn new(n: usize) -> Self {
        if n == 0 {
            panic!("Cannot create a Vector of size 0.")
        }
        Vector {
            capacity: n,
            values: Vec::with_capacity(n + 1),
            unused: PhantomData,
        }
    }
}

//----------------------------------------------------------------------------//
//  Container implementation.                                                 //
//----------------------------------------------------------------------------//

impl<K: Ord, V, R: Reference<V> + FromValue<V>> Insert<K, V, R>
    for Vector<K, V, R>
{
}

impl<K, V, R> Container<K, V, R> for Vector<K, V, R>
where
    K: Ord,
    R: Reference<V>,
{
    fn capacity(&self) -> usize {
        return self.capacity;
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

    fn pop(&mut self) -> Option<(K, R)> {
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

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
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

    fn take(&mut self, key: &K) -> Option<R> {
        match self.values.iter().position(|(k, _)| k == key) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i).1),
        }
    }
}

impl<K, V, R> Sequential<K, V, R> for Vector<K, V, R>
where
    K: Ord,
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

//----------------------------------------------------------------------------//
//  Vector iterator.                                                          //
//----------------------------------------------------------------------------//

impl<'a, K, V, R> Iter<'a, K, V, R> for Vector<K, V, R>
where
    K: 'a + Ord,
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

impl<'a, K, V, R> IterMut<'a, K, V, R> for Vector<K, V, R>
where
    K: 'a + Ord,
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

impl<K, V, R> IntoIterator for Vector<K, V, R>
where
    K: Ord,
    R: Reference<V>,
{
    type Item = (K, V);
    type IntoIter =
        std::iter::Map<std::vec::IntoIter<(K, R)>, fn((K, R)) -> (K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter().map(|(k, r)| (k, r.unwrap()))
    }
}

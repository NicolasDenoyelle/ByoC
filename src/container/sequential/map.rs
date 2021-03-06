use crate::container::{Container, Insert, Iter, IterMut, Packed, Sequential};
use crate::reference::{FromValue, Reference};
use std::marker::PhantomData;

//----------------------------------------------------------------------------//
//  key value map container.                                                  //
//----------------------------------------------------------------------------//

/// Ordered keys [`container`](../trait.Container.html).
///
/// `Map` container stores its element in a `BTreeMap`, thus leading to
/// fast cache lookups.
/// Evictions still require to walk the whole `Map` values and are O(n).
/// Lookups, removal and insertions (without evictions) are O(1).
///
/// ## Generics
///
/// * `K`: The type of key to use. Keys must implement `Copy` trait and `Ord`
/// trait to be work with `BTreeMap`.
/// * `V`: Value type stored in [cache reference](../reference/trait.Reference.html).
/// * `R`: A type of orderable [cache reference](../reference/trait.Reference.html).
///
/// ## Examples
///
/// ```
/// use std::string::String;
/// use cache::container::Container;
/// use cache::container::sequential::Map;
/// use cache::reference::{Reference, Default};
///
/// // container with only 1 element.
/// let mut c = Map::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", Default::new(4)).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", Default::new(12)).unwrap();
///
/// // The victim is the one with the greatest value
/// assert!(key == "second");
/// assert!(*value == 12);
/// ```
pub struct Map<K, V, R>
where
    K: Clone + Ord,
    R: Reference<V>,
{
    /// Container capacity
    capacity: usize,
    /// Map of references keys and values. Used for lookups.
    map: std::collections::BTreeMap<K, R>,
    unused: PhantomData<V>,
}

impl<K, V, R> Map<K, V, R>
where
    K: Clone + Ord,
    R: Reference<V>,
{
    pub fn new(n: usize) -> Self {
        Map {
            capacity: n,
            map: std::collections::BTreeMap::new(),
            unused: PhantomData,
        }
    }
}

//----------------------------------------------------------------------------//
//  Container implementation.                                                 //
//----------------------------------------------------------------------------//

impl<K: Clone + Ord, V, R: Reference<V> + FromValue<V>> Insert<K, V, R>
    for Map<K, V, R>
{
}

impl<K, V, R> Container<K, V, R> for Map<K, V, R>
where
    K: Clone + Ord,
    R: Reference<V>,
{
    fn capacity(&self) -> usize {
        return self.capacity.clone();
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn count(&self) -> usize {
        return self.map.len();
    }

    fn clear(&mut self) {
        self.map.clear()
    }

    fn take(&mut self, key: &K) -> Option<R> {
        self.map.remove(key)
    }

    fn pop(&mut self) -> Option<(K, R)> {
        let key = match self.map.iter().max_by(|x, y| x.1.cmp(&y.1)) {
            None => return None,
            Some((k, _)) => k.clone(),
        };
        Some((key.clone(), self.map.remove(&key).unwrap()))
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        match self.map.insert(key.clone(), reference) {
            Some(r) => Some((key, r)),
            None => {
                if self.capacity < self.map.len() {
                    self.pop()
                } else {
                    None
                }
            }
        }
    }
}

impl<K, V, R> Sequential<K, V, R> for Map<K, V, R>
where
    K: Clone + Ord,
    R: Reference<V>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        match self.map.get(key) {
            None => None,
            Some(r) => Some(r.deref()),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.map.get_mut(key) {
            None => None,
            Some(r) => {
                r.touch();
                Some(r.deref_mut())
            }
        }
    }
}

impl<K, V, R> Packed<K, V, R> for Map<K, V, R>
where
    K: Ord + Copy,
    R: Reference<V>,
{
}

//----------------------------------------------------------------------------//
//  Map iterators                                                             //
//----------------------------------------------------------------------------//

impl<K, V, R> IntoIterator for Map<K, V, R>
where
    K: Ord + Clone,
    R: Reference<V>,
{
    type Item = (K, V);
    type IntoIter = std::iter::Map<
        std::collections::btree_map::IntoIter<K, R>,
        fn((K, R)) -> (K, V),
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter().map(|(k, r)| (k, r.unwrap()))
    }
}

impl<'a, K, V, R> Iter<'a, K, V, R> for Map<K, V, R>
where
    K: 'a + Ord + Clone,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = std::iter::Map<
        std::collections::btree_map::IterMut<'a, K, R>,
        fn((&'a K, &'a mut R)) -> (&'a K, &'a V),
    >;
    fn iter(&'a mut self) -> Self::Iterator {
        self.map.iter_mut().map(|(k, r)| {
            r.touch();
            (k, r.deref())
        })
    }
}

impl<'a, K, V, R> IterMut<'a, K, V, R> for Map<K, V, R>
where
    K: 'a + Ord + Clone,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = std::iter::Map<
        std::collections::btree_map::IterMut<'a, K, R>,
        fn((&'a K, &'a mut R)) -> (&'a K, &'a mut V),
    >;
    fn iter_mut(&'a mut self) -> Self::Iterator {
        self.map.iter_mut().map(|(k, r)| {
            r.touch();
            (k, r.deref_mut())
        })
    }
}

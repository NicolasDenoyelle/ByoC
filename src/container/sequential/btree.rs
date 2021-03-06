use crate::container::{Container, Insert, Iter, IterMut, Packed, Sequential};
use crate::reference::{FromValue, Reference};
use crate::utils::ptr::OrdPtr;
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

//----------------------------------------------------------------------------//
// Ordered set of references and key value map.                               //
//----------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) with ordered keys and [references](../../reference/trait.Reference.html).
///
/// BTree is a container organized with binary tree structures.
/// Cache [references](../reference/trait.Reference.html) are kept in a binary tree
/// for fast search of
/// eviction candidates.
/// A binary tree map <key, value> is also maintained to enable
/// fast cache lookups.
/// As a result, insertions, removal, lookup and evictions are O(1).
/// However, this implementation require to store an additional pointer and
/// key per [cache reference](../reference/trait.Reference.html).
///
/// ## Generics:
///
/// * `K`: The type of key to use. Keys must implement `Copy` trait and `Ord`
/// trait to be work with `BTreeMap`.
/// * `V`: Value type stored in [cache reference](../reference/trait.Reference.html).
/// * `R`: A type of [cache reference](../reference/trait.Reference.html).
///
/// ## Examples
///
/// ```
/// use cache::container::Container;
/// use cache::container::sequential::BTree;
/// use cache::reference::{Reference, Default};
///
/// // container with only 1 element.
/// let mut c = BTree::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", Default::new(4u32)).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", Default::new(12u32)).unwrap();
///
/// // The victim is the second reference because its value is greater.
/// assert!(key == "second");
/// assert!(*value == 12u32);
/// ```
pub struct BTree<K, V, R>
where
    K: Copy + Ord,
    R: Reference<V>,
{
    /// Container capacity
    capacity: usize,
    /// Sparse vector of references.
    references: Vec<(K, R)>,
    /// Ordered set of references. Used for eviction.
    set: BTreeSet<(OrdPtr<R>, K)>,
    /// Map of references keys and index.
    map: BTreeMap<K, usize>,
    unused: PhantomData<V>,
}

impl<K, V, R> BTree<K, V, R>
where
    K: Copy + Ord,
    R: Reference<V>,
{
    pub fn new(n: usize) -> Self {
        BTree {
            capacity: n,
            references: Vec::with_capacity(n + 1),
            set: BTreeSet::new(),
            map: BTreeMap::new(),
            unused: PhantomData,
        }
    }
}

//----------------------------------------------------------------------------//
//  Container implementation.                                                 //
//----------------------------------------------------------------------------//

impl<K: Copy + Ord, V, R: Reference<V> + FromValue<V>> Insert<K, V, R>
    for BTree<K, V, R>
{
}

impl<K, V, R> Container<K, V, R> for BTree<K, V, R>
where
    K: Copy + Ord,
    R: Reference<V>,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn count(&self) -> usize {
        return self.references.len();
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn clear(&mut self) {
        self.map.clear();
        self.set.clear();
        self.references.clear();
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        match self.map.get(&key) {
            Some(j) => {
                assert!(self
                    .set
                    .remove(&(OrdPtr::new(&self.references[*j].1), key)));
                self.references.push((key, reference));
                let ret = Some(self.references.swap_remove(*j));
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[*j].1), key)));
                ret
            }
            None => {
                self.references.push((key, reference));
                let n = self.references.len() - 1;
                assert!(self.map.insert(key, n).is_none());
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[n].1), key)));
                if self.references.len() > self.capacity {
                    self.pop()
                } else {
                    None
                }
            }
        }
    }

    fn pop(&mut self) -> Option<(K, R)> {
        let k = match self.set.iter().rev().next() {
            None => return None,
            Some((_, k)) => k.clone(),
        };

        let n = self.references.len() - 1;
        let j = self.map.remove(&k).unwrap();
        assert!(self.set.remove(&(OrdPtr::new(&self.references[j].1), k)));

        if j != n {
            let (k_last, r_last) = {
                let (k, r) = self.references.iter().rev().next().unwrap();
                (k.clone(), OrdPtr::new(r))
            };
            assert!(self.set.remove(&(r_last, k_last)));
            self.map.insert(k_last, j);
            let ret = self.references.swap_remove(j);
            assert!(self
                .set
                .insert((OrdPtr::new(&self.references[j].1), k_last)));
            Some(ret)
        } else {
            Some(self.references.swap_remove(j))
        }
    }

    fn take(&mut self, key: &K) -> Option<R> {
        match self.map.remove(key) {
            None => None,
            Some(i) => {
                let n = self.references.len() - 1;
                assert!(self
                    .set
                    .remove(&(OrdPtr::new(&self.references[i].1), *key)));
                if i != n {
                    let (k_last, r_last) = {
                        let (k, r) =
                            self.references.iter().rev().next().unwrap();
                        (k.clone(), OrdPtr::new(r))
                    };
                    assert!(self.set.remove(&(r_last, k_last)));
                    self.map.insert(k_last, i);
                    let (_, reference) = self.references.swap_remove(i);
                    assert!(self
                        .set
                        .insert((OrdPtr::new(&self.references[i].1), k_last)));
                    Some(reference)
                } else {
                    let (_, reference) = self.references.swap_remove(i);
                    Some(reference)
                }
            }
        }
    }
}

impl<K, V, R> Sequential<K, V, R> for BTree<K, V, R>
where
    K: Copy + Ord,
    R: Reference<V>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        match self.get_mut(key) {
            None => None,
            Some(v) => Some(v),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.map.get(key) {
            None => None,
            Some(i) => {
                assert!(self
                    .set
                    .remove(&(OrdPtr::new(&self.references[*i].1), *key)));
                self.references[*i].1.touch();
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[*i].1), *key)));
                Some(self.references[*i].1.deref_mut())
            }
        }
    }
}

impl<K, V, R> Packed<K, V, R> for BTree<K, V, R>
where
    K: Ord + Copy,
    R: Reference<V>,
{
}

//----------------------------------------------------------------------------//
// BTree iterator                                                             //
//----------------------------------------------------------------------------//

impl<K, V, R> IntoIterator for BTree<K, V, R>
where
    K: Ord + Copy,
    R: Reference<V>,
{
    type Item = (K, V);
    type IntoIter =
        std::iter::Map<std::vec::IntoIter<(K, R)>, fn((K, R)) -> (K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.references.into_iter().map(|(k, r)| (k, r.unwrap()))
    }
}

/// Iterator of ref mut BTree [container](../trait.Container.html).
pub struct BTreeIterator<'a, K, V, R>
where
    K: Ord + Copy,
    R: Reference<V>,
{
    set: &'a mut BTreeSet<(OrdPtr<R>, K)>,
    iter: std::slice::IterMut<'a, (K, R)>,
    unused: PhantomData<V>,
}

impl<'a, K, V, R> Iterator for BTreeIterator<'a, K, V, R>
where
    K: Ord + Copy,
    V: 'a,
    R: Reference<V>,
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some((k, r)) => {
                assert!(self.set.remove(&(OrdPtr::new(r), *k)));
                r.touch();
                assert!(self.set.insert((OrdPtr::new(r), *k)));
                Some((k, r.deref_mut()))
            }
        }
    }
}

impl<'a, K, V, R> IterMut<'a, K, V, R> for BTree<K, V, R>
where
    K: 'a + Ord + Copy,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = BTreeIterator<'a, K, V, R>;
    fn iter_mut(&'a mut self) -> Self::Iterator {
        BTreeIterator {
            set: &mut self.set,
            iter: self.references.iter_mut(),
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, R> Iter<'a, K, V, R> for BTree<K, V, R>
where
    K: 'a + Ord + Copy,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator = std::iter::Map<
        BTreeIterator<'a, K, V, R>,
        fn((&'a K, &'a mut V)) -> (&'a K, &'a V),
    >;
    fn iter(&'a mut self) -> Self::Iterator {
        BTreeIterator {
            set: &mut self.set,
            iter: self.references.iter_mut(),
            unused: PhantomData,
        }
        .map(|(k, v)| (k, v))
    }
}

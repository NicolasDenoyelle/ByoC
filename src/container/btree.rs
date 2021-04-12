use crate::container::Container;
use crate::marker::Packed;
use crate::utils::ptr::OrdPtr;
use std::collections::{BTreeMap, BTreeSet};

//------------------------------------------------------------------------//
// Ordered set of references and key value map.                           //
//------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) with ordered keys and
/// [references](../../reference/trait.Reference.html).
///
/// BTree is a container organized with binary tree structures.
/// Values are kept in a binary tree for fast search of eviction candidates.
/// A binary tree map <key, value> is also maintained to enable
/// fast cache lookups.
/// As a result, insertions, removal, lookup and evictions are O(log(n)).
/// However, this implementation require to store an additional pointer and
/// key per key/value pair.
///
/// ## Generics:
///
/// * `K`: The type of key to use.
/// * `V`: Value type stored.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, BTree};
///
/// // container with only 1 element.
/// let mut c = BTree::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", 4).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", 12).unwrap();
///
/// // The victim is the first reference because eviction happens before
/// // insertion.
/// assert!(key == "first");
/// assert!(value == 4);
/// ```
pub struct BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    /// Container capacity
    capacity: usize,
    /// Sparse vector of references.
    references: Vec<(K, V)>,
    /// Ordered set of references. Used for eviction.
    set: BTreeSet<(OrdPtr<V>, K)>,
    /// Map of references keys and index.
    map: BTreeMap<K, usize>,
}

impl<K, V> BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    pub fn new(n: usize) -> Self {
        BTree {
            capacity: n,
            references: Vec::with_capacity(n + 1),
            set: BTreeSet::new(),
            map: BTreeMap::new(),
        }
    }
}

//------------------------------------------------------------------------//
//  Container implementation.                                             //
//------------------------------------------------------------------------//

impl<K, V> Container<K, V> for BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn count(&self) -> usize {
        return self.references.len();
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        self.map.clear();
        self.set.clear();
        self.references.drain(..).collect()
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn clear(&mut self) {
        self.map.clear();
        self.set.clear();
        self.references.clear();
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
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
                let out = if self.references.len() >= self.capacity {
                    self.pop()
                } else {
                    None
                };

                self.references.push((key, reference));
                let n = self.references.len() - 1;
                assert!(self.map.insert(key, n).is_none());
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[n].1), key)));
                out
            }
        }
    }

    fn pop(&mut self) -> Option<(K, V)> {
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

    fn take(&mut self, key: &K) -> Option<V> {
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
                    assert!(self.set.insert((
                        OrdPtr::new(&self.references[i].1),
                        k_last
                    )));
                    Some(reference)
                } else {
                    let (_, reference) = self.references.swap_remove(i);
                    Some(reference)
                }
            }
        }
    }
}

impl<K, V> Packed<K, V> for BTree<K, V>
where
    K: Ord + Copy,
    V: Ord,
{
}

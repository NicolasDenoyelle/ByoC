use crate::private::ptr::OrdPtr;
use crate::{BuildingBlock, GetMut, Ordered};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Ordered set of references and key value map.                           //
//------------------------------------------------------------------------//

/// [`BuildingBlock`](../trait.BuildingBlock.html) with ordered keys and
/// values.
///
/// BTree is a container organized with binary tree structures for keys
/// and values. Keys are kept in a binary tree for fast lookups.
/// Values are kept in a binary tree for fast search of eviction candidates.
/// Since keys are ordered, this container will not allow several matching
/// keys in the container. However, it can store equal values.
///
/// BTree does not implement [`Get`](../trait.Get.html) trait because
/// accessing values, even in a non exclusive way, may change their
/// relative order and break the way values are stored in a binary tree.
///
/// See
/// [`BuildingBlock methods implementation`](struct.BTree.html#impl-BuildingBlock%3C%27a%2C%20K%2C%20V%3E)
/// for behavior on `push()` and `pop()`.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::BTree;
///
/// // BTree with 3 elements capacity.
/// let mut c = BTree::new(3);
///
/// // BuildingBlock as room for 2 elements and returns an empty vector.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4), ("second", 2)]).pop().is_none());
///
/// // Insertion of existing keys are rejected and elements not fitting
/// // in the container are also rejected.
/// let out = c.push(vec![("second", 4), ("third", 3), ("fourth", 4)]);
/// // Already in the container.
/// assert_eq!(out[0].0, "second");
/// // Overflow
/// assert_eq!(out[1].0, "fourth");
/// assert_eq!(out.len(), 2);
///
/// // BTree pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "first");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "third");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "second");
/// ```
pub struct BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    // BuildingBlock capacity
    capacity: usize,
    // Sparse vector of references.
    references: Vec<(K, V)>,
    // Ordered set of references. Used for eviction.
    set: BTreeSet<(OrdPtr<V>, K)>,
    // Map of references keys and index.
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
            references: Vec::with_capacity(n),
            set: BTreeSet::new(),
            map: BTreeMap::new(),
        }
    }
}

impl<'a, K, V> BuildingBlock<'a, K, V> for BTree<K, V>
where
    K: 'a + Copy + Ord + std::fmt::Debug,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn count(&self) -> usize {
        return self.references.len();
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.map.clear();
        self.set.clear();
        Box::new(
            self.references
                .drain(..)
                .collect::<Vec<(K, V)>>()
                .into_iter(),
        )
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
    /// returned. This container does not accept keys matching keys
    /// already inside the container and will return the corresponding
    /// key/value pairs.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut out = Vec::new();
        let mut n = self.references.len();

        for (key, value) in values.into_iter() {
            if n >= self.capacity {
                println!("Reject {:?} because capacity exceeded.", key);
                out.push((key, value));
            } else if let Some(_) = self.map.get(&key) {
                println!("Reject {:?} because already stored.", key);
                out.push((key, value))
            } else {
                println!("Insert {:?}.", key);
                self.references.push((key, value));
                assert!(self.map.insert(key, n).is_none());
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[n].1), key)));
                n += 1;
            }
        }
        out
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../policy/trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// retrieving the last values stored in a binary tree.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut ret = Vec::new();

        for _ in 0..n {
            let k = match self.set.iter().rev().next() {
                None => break,
                Some((_, k)) => k.clone(),
            };

            let n = self.references.len() - 1;
            let j = self.map.remove(&k).unwrap();
            assert!(self
                .set
                .remove(&(OrdPtr::new(&self.references[j].1), k)));

            let e = if j != n {
                let (k_last, r_last) = {
                    let (k, r) =
                        self.references.iter().rev().next().unwrap();
                    (k.clone(), OrdPtr::new(r))
                };
                assert!(self.set.remove(&(r_last, k_last)));
                self.map.insert(k_last, j);
                let ret = self.references.swap_remove(j);
                assert!(self
                    .set
                    .insert((OrdPtr::new(&self.references[j].1), k_last)));
                ret
            } else {
                self.references.swap_remove(j)
            };
            ret.push(e);
        }
        ret
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
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
                    let (key, reference) = self.references.swap_remove(i);
                    assert!(self.set.insert((
                        OrdPtr::new(&self.references[i].1),
                        k_last
                    )));
                    Some((key, reference))
                } else {
                    let (key, reference) = self.references.swap_remove(i);
                    Some((key, reference))
                }
            }
        }
    }
}

// Make this container usable with a policy.
impl<K: Ord + Copy, V: Ord> Ordered<V> for BTree<K, V> {}

//------------------------------------------------------------------------//
//  GetMut trait implementation
//------------------------------------------------------------------------//

pub struct BTreeCell<K: Copy + Ord, V: Ord> {
    key: K,
    value: OrdPtr<V>,
    // Where to reinsert element on drop.
    set: *mut BTreeSet<(OrdPtr<V>, K)>,
}

impl<K: Copy + Ord, V: Ord> Deref for BTreeCell<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<K: Copy + Ord, V: Ord> DerefMut for BTreeCell<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}

impl<K: Copy + Ord, V: Ord> Drop for BTreeCell<K, V> {
    fn drop(&mut self) {
        unsafe {
            assert!(self
                .set
                .as_mut()
                .unwrap()
                .insert((self.value.clone(), self.key)));
        }
    }
}

impl<K: Copy + Ord, V: Ord> GetMut<K, V, BTreeCell<K, V>> for BTree<K, V> {
    unsafe fn get_mut(&mut self, key: &K) -> Option<BTreeCell<K, V>> {
        match self.map.get(key) {
            None => None,
            Some(i) => {
                let (_, value) = self.references.get(*i).unwrap();
                let value = OrdPtr::new(value);
                let vk = (value, *key);
                assert!(self.set.remove(&vk));
                let (value, key) = vk;
                Some(BTreeCell {
                    key: key,
                    value: value,
                    set: &mut self.set,
                })
            }
        }
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::policy::tests::test_ordered;
    use crate::tests::{test_building_block, test_get_mut};

    #[test]
    fn building_block() {
        test_building_block(BTree::new(0));
        test_building_block(BTree::new(10));
        test_building_block(BTree::new(100));
    }

    #[test]
    fn ordered() {
        test_ordered(BTree::new(0));
        test_ordered(BTree::new(10));
        test_ordered(BTree::new(100));
    }

    #[test]
    fn get() {
        test_get_mut(BTree::new(0));
        test_get_mut(BTree::new(10));
        test_get_mut(BTree::new(100));
    }
}

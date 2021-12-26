use crate::{BuildingBlock, GetMut, Ordered, Prefetch};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::rc::Rc;

//------------------------------------------------------------------------//
// Ordered set of references and key value map.                           //
//------------------------------------------------------------------------//

/// Building block with ordered keys and values.
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
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::BTree;
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
    references: Vec<(K, Rc<V>)>,
    // Ordered set of references. Used for eviction.
    set: BTreeSet<(Rc<V>, K)>,
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
    K: 'a + Copy + Ord,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        self.references.len()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.map.clear();
        self.set.clear();
        Box::new(
            self.references
                .drain(..)
                .map(|(k, v)| {
                    (k, Rc::try_unwrap(v).map_err(|_| panic!("")).unwrap())
                })
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
                out.push((key, value));
            } else if self.map.get(&key).is_some() {
                out.push((key, value))
            } else {
                let value = Rc::new(value);
                let _value = Rc::clone(&value);
                self.references.push((key, value));
                assert!(self.map.insert(key, n).is_none());
                assert!(self.set.insert((_value, key)));
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
    /// [`Ordered`](../trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// retrieving the last values stored in a binary tree.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut ret = Vec::new();

        for _ in 0..n {
            let k = match self.set.iter().rev().next() {
                None => break,
                Some((_, k)) => *k,
            };

            let n = self.references.len() - 1;
            let j = self.map.remove(&k).unwrap();
            assert!(self
                .set
                .remove(&(Rc::clone(&self.references[j].1), k)));

            // If we don't remove the last element, we need to update the
            // position of the element it is swapped with.
            if j != n {
                let (k, _) = self
                    .references
                    .iter()
                    .rev()
                    .map(|(k, r)| (*k, Rc::clone(r)))
                    .next()
                    .unwrap();
                self.map.insert(k, j);
            }

            let (k, r) = self.references.swap_remove(j);
            ret.push((
                k,
                Rc::try_unwrap(r).map_err(|_| panic!("")).unwrap(),
            ));
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
                    .remove(&(Rc::clone(&self.references[i].1), *key)));
                if i != n {
                    let (k, _) = self
                        .references
                        .iter()
                        .rev()
                        .map(|(k, r)| (*k, Rc::clone(r)))
                        .next()
                        .unwrap();
                    self.map.insert(k, i);
                }
                let (k, r) = self.references.swap_remove(i);
                Some((
                    k,
                    Rc::try_unwrap(r).map_err(|_| panic!("")).unwrap(),
                ))
            }
        }
    }
}

// Make this container usable with a policy.
impl<K: Ord + Copy, V: Ord> Ordered<V> for BTree<K, V> {}

//------------------------------------------------------------------------//
//  GetMut trait implementation
//------------------------------------------------------------------------//

/// Cell representing a writable value inside a
/// [`BTree`](struct.BTree.html).
///
/// This value inside this cell is taken out of the container and written
/// back in it when the cell is dropped.
pub struct BTreeCell<K: Copy + Ord, V: Ord> {
    kv: Option<(K, V)>,
    set: NonNull<BTree<K, V>>,
}

impl<K: Copy + Ord, V: Ord> Deref for BTreeCell<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.kv.as_ref().unwrap().1
    }
}

impl<K: Copy + Ord, V: Ord> DerefMut for BTreeCell<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kv.as_mut().unwrap().1
    }
}

impl<K: Copy + Ord, V: Ord> Drop for BTreeCell<K, V> {
    fn drop(&mut self) {
        let set = unsafe { self.set.as_mut() };
        let kv = self.kv.take().unwrap();
        assert!(set.push(vec![kv]).pop().is_none());
    }
}

impl<K: Copy + Ord, V: Ord> GetMut<K, V, BTreeCell<K, V>> for BTree<K, V> {
    unsafe fn get_mut(&mut self, key: &K) -> Option<BTreeCell<K, V>> {
        self.take(key).map(|(key, value)| BTreeCell {
                kv: Some((key, value)),
                set: NonNull::new(self).unwrap(),
            })
    }
}

impl<'a, K: 'a + Copy + Ord, V: 'a + Ord> Prefetch<'a, K, V>
    for BTree<K, V>
{
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::{test_building_block, test_get_mut, test_ordered};

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

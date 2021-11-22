use crate::private::ptr::OrdPtr;
use crate::BuildingBlock;
use std::collections::{BTreeMap, BTreeSet};

//------------------------------------------------------------------------//
// Ordered set of references and key value map.                           //
//------------------------------------------------------------------------//

/// [`BuildingBlock`](../trait.BuildingBlock.html) with ordered keys and values.
///
/// BTree is a container organized with a binary tree structures.
/// Keys are kept in a binary tree for fast lookups.
/// Values are kept in a binary tree for fast search of eviction candidates.
/// Since keys are ordered, this container will not allow several matching
/// keys in the container. However, it can store similar values paired with
/// different keys.
/// Insertions, removal, lookup and evictions are O(log(n)).
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::BTree;
///
/// // container with only 1 element.
/// let mut c = BTree::new(1);
///
/// // BuildingBlock as room for first element and returns None.
/// assert!(c.push(vec![("first", 4)]).pop().is_none());
///
/// // BuildingBlock is full and pops a inserted element.
/// let (key, value) = c.push(vec![("second", 12)]).pop().unwrap();
/// assert!(key == "second");
/// assert!(value == 12);
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
            references: Vec::with_capacity(n + 1),
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

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = std::cmp::min(
            self.capacity - self.references.len(),
            values.len(),
        );
        let mut out = values.split_off(n);

        for (key, value) in values.into_iter() {
            match self.map.get(&key) {
                Some(_) => out.push((key, value)),
                None => {
                    self.references.push((key, value));
                    let n = self.references.len() - 1;
                    assert!(self.map.insert(key, n).is_none());
                    assert!(self.set.insert((
                        OrdPtr::new(&self.references[n].1),
                        key
                    )));
                }
            }
        }
        out
    }

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

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::container::tests::test_container;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        test_building_block(BTree::new(0));
        test_building_block(BTree::new(10));
        test_building_block(BTree::new(100));
    }

    #[test]
    fn container() {
        test_container(BTree::new(0));
        test_container(BTree::new(10));
        test_container(BTree::new(100));
    }
}

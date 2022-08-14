use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::vec::Vec;

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
    pub(super) capacity: usize,
    // Sparse vector of references.
    pub(super) references: Vec<(K, Rc<V>)>,
    // Ordered set of references. Used for eviction.
    pub(super) set: BTreeSet<(Rc<V>, K)>,
    // Map of references keys and index.
    pub(super) map: BTreeMap<K, usize>,
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

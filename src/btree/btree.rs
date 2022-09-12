use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::vec::Vec;

/// In-memory container with ordered keys and values.
///
/// [`BTree`] is a container organized with binary tree structures for keys
/// and values. Keys are kept in a binary tree for fast lookups.
/// Values are kept in a binary tree for fast search of eviction candidates.
/// This container cannot contain matching pairs of a key and a value.
/// However, it can store matching values or matching keys.
/// Additionally, because [`BTree`] is an ordered container,
/// it is not safe to store keys or values for which the order may change
/// through time. Specifically, using [`Lru`](./policy/struct.Lru.html)
/// or [`Lrfu`](./policy/struct.Lrfu.html) policies is not safe.
///
/// The number of elements fitting in the container is computed similarly
/// to [`Array`](struct.Array.html) container as the sum of its
/// [elements stack size](struct.Array.html#method.element_size).
/// However, elements occupy a larger size since there are several structures
/// involved to store elements, to order keys and to order values.
///
/// * Insertion complexity is `$O(log(n))$`.
/// The whole array is walked to look for matching keys and avoid collisions.
/// * Removal complexity is `$O(log(n))$`.
/// The whole array is walked to look for matching keys.
/// * Eviction complexity is `$O(log(n))$`.
/// The whole array is sorted to remove the top `k` elements.
/// * Keys lookup complexity is `$O(log(n))$`.
/// * Capacity and count queries are `$O(1)$`.
///
/// [`BTree`] does not implement [`Get`](trait.Get.html) trait because
/// accessing modifying values is likely to modify their order and break the
/// container logic.
///
/// This container implements the [`Ordered`](../policy/trait.Ordered.html)
/// (although it cannot be safely used with a [policy](policy/index.html))
/// because eviction will evict the top `k` elements.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::BTree;
///
/// // BTree with 3 elements capacity.
/// let mut c = BTree::new(3 * BTree::<&str, u32>::element_size());
///
/// // BuildingBlock as room for 2 elements and returns an empty vector.
/// // No element is rejected.
/// assert!(c.push(vec![("first", 4), ("second", 2)]).pop().is_none());
///
/// // Insertion of existing keys is rejected and elements not fitting
/// // in the container are returned.
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
///
/// [`BTree`] can also be built from a
/// [builder pattern](builder/struct.Builder.html#method.btree) and a
/// [configuration](config/struct.BTreeConfig.html).
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
        let num_elements = n / Self::element_size();
        BTree {
            capacity: num_elements,
            references: Vec::with_capacity(num_elements),
            set: BTreeSet::new(),
            map: BTreeMap::new(),
        }
    }

    /// The estimated size of one element in the container.
    pub fn element_size() -> usize {
        std::mem::size_of::<V>()
            + std::mem::size_of::<Rc<V>>() * 2
            + std::mem::size_of::<K>() * 3
            + std::mem::size_of::<usize>()
    }
}

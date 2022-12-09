use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// In-memory container with ordered keys and values.
///
/// [`BTree`] is a container organized with binary tree structures for keys
/// and values. Keys are kept in a binary tree for fast lookups.
/// Values are kept in a binary tree for fast search of eviction candidates.
/// When evicting candidates, key/value pairs with the greatest values are
/// evicted first.
///
/// Elements that go into a [`BTree`] are sized with a function
/// `element_size()` set with the method
/// [`with_element_size()`](struct.BTree.html#method.with_element_size) such
/// that the sum of the sizes of [`BTree`] elements never exceed its set
/// capacity. The default size for a key/value pair element is `1` and therefore
/// the container capacity in this circumstance is the number of elements it can
/// contain.
///
/// This container cannot contain matching pairs of couple (key, value).
/// However, it can store matching values or matching keys.
/// Additionally, because [`BTree`] is an ordering container,
/// it is not safe to store keys or values for which the order may change
/// through time.
///
/// See [`BuildingBlock` implementation](struct.BTree.html#impl-BuildingBlock)
/// for more detail on how does the container operates.
///
/// Elements within the array can be accessed with the [`Get`](trait.Get.html)
/// and [`GetMut`](trait.GetMut.html) traits.
///
/// ## Examples
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
/// assert!(c.push(vec![("first", 1), ("second", 2)]).pop().is_none());
///
/// // Pushing pops out duplicates. If there is enough size for the rest, the
/// // rest will fit.
/// let out = c.push(vec![("first", 2), ("third", 3)]);
/// assert_eq!(out.len(), 1);
/// assert_eq!(out[0].0, "first");
///
/// // Overflowing pushes will pop enough room for new elements:
/// let out = c.push(vec![("fourth", 4)]);
/// assert_eq!(out.len(), 1);
/// // Popped element is the greatest value already present before push.
/// assert_eq!(out[0].0, "third");
///
/// // BTree pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "fourth");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "second");
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, "first");
/// assert!(c.pop(1).is_empty());
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
    pub(super) total_size: usize,
    pub(super) element_size: fn((&K, &V)) -> usize,
    // Ordering set of references. Used for eviction.
    pub(super) set: BTreeSet<(Rc<V>, K)>,
    // Map of references keys and index.
    pub(super) map: BTreeMap<K, Rc<V>>,
}

impl<K, V> BTree<K, V>
where
    K: Copy + Ord,
    V: Ord,
{
    /// Create a new [`BTree`] container with `size`
    /// [`capacity`](struct.BTree.html#method.capacity).
    ///
    /// The meaning of this capacity depends on the `element_size` function
    /// set with
    /// [`with_element_size()`](struct.BTree.html#method.with_element_size).
    /// The default is to set every elements size to `1usize` and therefore,
    /// `size` stands for the maximum number of elements fitting in the
    /// [`BTree`].
    pub fn new(size: usize) -> Self {
        BTree {
            capacity: size,
            total_size: 0,
            element_size: |_| 1,
            set: BTreeSet::new(),
            map: BTreeMap::new(),
        }
    }

    /// Set how [`BTree`] elements size is computed.
    ///
    /// Whenever an element is inserted or removed from the [`BTree`],
    /// its size is compared with the container capacity and its remaining
    /// space to decide respectively, whether the element can be inserted or
    /// how much space does it leaves in the container.
    /// This function decides how to compute each element size and therefore
    /// it also decides of the meaning of the container
    /// [`capacity`](struct.BTree.html#method.capacity).
    pub fn with_element_size(
        mut self,
        element_size: fn((&K, &V)) -> usize,
    ) -> Self {
        if self.total_size > 0 {
            panic!("It is not allowed to set a non empty BTree container element_size method.")
        }
        self.element_size = element_size;
        self
    }

    pub(super) fn as_value(rc: Rc<V>) -> V {
        Rc::try_unwrap(rc).map_err(|_| panic!("")).unwrap()
    }

    pub(super) fn insert_values_unchecked(
        &mut self,
        elements: Vec<(K, V)>,
    ) {
        for (key, value) in elements.into_iter() {
            let value = Rc::new(value);
            self.set.insert((Rc::clone(&value), key));
            self.map.insert(key, value);
        }
    }
}

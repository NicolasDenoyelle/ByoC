use crate::builder::Build;
use crate::BTree;
use std::marker::PhantomData;

/// `BTree` builder.
///
/// This builder can be consumed later to spawn an
/// [`BTree`](../../struct.BTree.html) container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::builder::builders::BTreeBuilder;
///
/// let mut btree = BTreeBuilder::new(2).build();
/// btree.push(vec![(1, 2)]);
/// ```
pub struct BTreeBuilder<K: Ord + Copy, V: Ord> {
    capacity: usize,
    unused: PhantomData<(K, V)>,
}

impl<K: Ord + Copy, V: Ord> BTreeBuilder<K, V> {
    pub fn new(capacity: usize) -> Self {
        BTreeBuilder {
            capacity,
            unused: PhantomData,
        }
    }
}

impl<K: Ord + Copy, V: Ord> Clone for BTreeBuilder<K, V> {
    fn clone(&self) -> Self {
        BTreeBuilder {
            capacity: self.capacity,
            unused: PhantomData,
        }
    }
}

impl<K: Copy + Ord, V: Ord> Build<BTree<K, V>> for BTreeBuilder<K, V> {
    fn build(self) -> BTree<K, V> {
        BTree::new(self.capacity)
    }
}

use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::BTree;
use std::marker::PhantomData;

/// `BTree` builder.
///
/// This builder can be consumed later to spawn an
/// [`BTree`](../../struct.BTree.html) container.
///
/// # Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::BTreeBuilder;
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

impl<K: Ord + Copy, V: Ord> Associative<BTree<K, V>>
    for BTreeBuilder<K, V>
{
}

impl<K: Ord + Copy, V: Ord> Sequential<BTree<K, V>>
    for BTreeBuilder<K, V>
{
}

impl<K: Ord + Copy, V: Ord> Profiler<BTree<K, V>> for BTreeBuilder<K, V> {}

impl<K: Ord + Copy, V: Ord> Policy<BTree<K, V>> for BTreeBuilder<K, V> {}
impl<K, V, R, RB> Multilevel<BTree<K, V>, R, RB> for BTreeBuilder<K, V>
where
    K: Ord + Copy,
    V: Ord,
    RB: Builder<R>,
{
}

impl<K: Copy + Ord, V: Ord> Builder<BTree<K, V>> for BTreeBuilder<K, V> {
    fn build(self) -> BTree<K, V> {
        BTree::new(self.capacity)
    }
}

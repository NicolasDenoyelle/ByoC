use crate::container::BTree;
use crate::policy::{Reference, ReferenceFactory};
use crate::builder::{
    Builder, ForwardBuilder, PolicyBuilder,
};
use std::marker::PhantomData;

pub struct BTreeBuilder<K: Ord + Copy, V: Ord> {
    capacity: usize,
		unused: PhantomData<(K,V)>,
}

impl<K: Ord + Copy, V: Ord> BTreeBuilder<K,V> {
    pub fn forward<R, RB: Builder<R>>(
        self,
    ) -> ForwardBuilder<BTree<K, V>, BTreeBuilder<K,V>, R, RB> {
        ForwardBuilder::new(self)
    }

    pub fn with_policy<
        R: Reference<V>,
        F: ReferenceFactory<V, R>,
    >(
        self,
        policy: F,
    ) -> PolicyBuilder<BTree<K, V>, V, R, F, BTreeBuilder<K,V>> {
        PolicyBuilder::new(self, policy)
    }
}

impl<K: Copy + Ord, V: Ord> Builder<BTree<K, V>> for BTreeBuilder<K,V> {
    fn build(self) -> BTree<K, V> {
        BTree::new(self.capacity)
    }
}

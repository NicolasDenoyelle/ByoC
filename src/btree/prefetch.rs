use super::BTree;
use crate::Prefetch;

impl<'a, K: 'a + Copy + Ord, V: 'a + Ord> Prefetch<'a, K, V>
    for BTree<K, V>
{
}

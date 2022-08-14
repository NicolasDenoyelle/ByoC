use super::BTree;
use crate::Ordered;

// Make this container usable with a policy.
impl<K: Ord + Copy, V: Ord> Ordered<V> for BTree<K, V> {}

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::test_ordered;
    #[test]
    fn ordered() {
        test_ordered(BTree::new(0));
        test_ordered(BTree::new(10));
        test_ordered(BTree::new(100));
    }
}

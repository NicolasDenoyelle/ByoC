use cache::container::BTree;
mod container;

#[test]
fn btree_container_test_0() {
    container::test_container(BTree::new(0), true);
}

#[test]
fn btree_container_test_small() {
    container::test_container(BTree::new(100), true);
}

use cache::container::sequential::BTree;
mod packed;
mod sequential;

#[test]
fn btree_container_test_0() {
    packed::test_container(BTree::new(0));
}

#[test]
fn btree_container_test_small() {
    packed::test_container(BTree::new(10));
}

#[test]
fn btree_container_test_large() {
    packed::test_container(BTree::new(1000));
}

#[test]
fn btree_sequential_test_0() {
    sequential::test_sequential(BTree::new(0));
}

#[test]
fn btree_sequential_test_small() {
    sequential::test_sequential(BTree::new(100));
}

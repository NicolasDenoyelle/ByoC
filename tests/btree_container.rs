use cache::building_block::container::BTree;
mod container;

#[test]
fn container_test() {
    container::test_container(BTree::new(0), true);
    container::test_container(BTree::new(100), true);
}

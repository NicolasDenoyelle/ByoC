mod concurrent;
mod container;
mod get;

use cache::building_block::container::Vector;
use cache::building_block::wrapper::Sequential;

#[test]
fn container_test() {
    container::test_container(Sequential::new(Vector::new(0)), true);
    container::test_container(Sequential::new(Vector::new(100)), true);
}

#[test]
fn get_test() {
    get::test_get(Sequential::new(Vector::new(0)));
    get::test_get(Sequential::new(Vector::new(100)));
}

#[test]
fn concurrent_test() {
    concurrent::test_concurrent(Sequential::new(Vector::new(0)), 64);
    concurrent::test_concurrent(Sequential::new(Vector::new(100)), 64);
}

mod container;
mod get;
use cache::building_block::container::Vector;

#[test]
fn container_test() {
    container::test_container(Vector::new(0), true);
    container::test_container(Vector::new(100), true);
}

#[test]
fn get_test() {
    get::test_get(Vector::new(0));
    get::test_get(Vector::new(100));
}

mod container;
use cache::container::Vector;

#[test]
fn container_test_0() {
    container::test_container(Vector::new(0), true);
}

#[test]
fn container_test_100() {
    container::test_container(Vector::new(100), true);
}

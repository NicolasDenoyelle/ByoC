use cache::container::sequential::Vector;
mod container;

#[test]
fn vector_container_test() {
    container::test_container(Vector::new(10), 100);
    container::test_container(Vector::new(100), 10);
}

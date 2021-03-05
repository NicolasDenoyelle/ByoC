mod container;
mod sequential;
use cache::container::sequential::Vector;

#[test]
fn vector_container_test_0() {
    container::test_container(Vector::new(0));
}

#[test]
fn vector_container_test_small() {
    container::test_container(Vector::new(10));
}

#[test]
fn vector_container_test_large() {
    container::test_container(Vector::new(1000));
}

#[test]
fn vector_sequential_test_0() {
    sequential::test_sequential(Vector::new(0));
}

#[test]
fn vector_sequential_test_small() {
    sequential::test_sequential(Vector::new(100));
}

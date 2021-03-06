mod packed;
mod sequential;
use cache::container::sequential::Vector;

#[test]
fn filemap_container_test_0() {
    packed::test_container(Vector::new(0));
}

#[test]
fn filemap_container_test_small() {
    packed::test_container(Vector::new(100));
}

#[test]
fn vector_container_test_large() {
    packed::test_container(Vector::new(1000));
}

#[test]
fn vector_sequential_test_0() {
    sequential::test_sequential(Vector::new(0));
}

#[test]
fn vector_sequential_test_small() {
    sequential::test_sequential(Vector::new(100));
}

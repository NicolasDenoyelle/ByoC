use cache::container::sequential::{Stack, Vector};
mod container;
mod sequential;

#[test]
fn stack_container_test_0() {
    container::test_container(Stack::new(Vector::new(0), Vector::new(0)));
    container::test_container(Stack::new(Vector::new(0), Vector::new(10)));
    container::test_container(Stack::new(Vector::new(10), Vector::new(0)));
}

#[test]
fn stack_container_test_small() {
    container::test_container(Stack::new(Vector::new(10), Vector::new(10)));
}

#[test]
fn stack_container_test_large() {
    container::test_container(Stack::new(Vector::new(10), Vector::new(1000)));
}

#[test]
fn vector_sequential_test_0() {
    sequential::test_sequential(Stack::new(Vector::new(0), Vector::new(0)));
}

#[test]
fn vector_sequential_test_small() {
    sequential::test_sequential(Stack::new(Vector::new(10), Vector::new(100)));
}

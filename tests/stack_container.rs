use cache::container::sequential::{Stack, Vector};
mod packed;

#[test]
fn stack_container_test_0() {
    packed::test_container(Stack::new(Vector::new(0), Vector::new(0)));
    packed::test_container(Stack::new(Vector::new(0), Vector::new(10)));
    packed::test_container(Stack::new(Vector::new(10), Vector::new(0)));
}

#[test]
fn stack_container_test_small() {
    packed::test_container(Stack::new(Vector::new(10), Vector::new(100)));
}

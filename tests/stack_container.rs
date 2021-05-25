use cache::container::{Stack, Vector};
mod container;

#[test]
fn stack_container_test_0() {
    container::test_container(
        Stack::new(Vector::new(0), Vector::new(0)),
        true,
    );
    container::test_container(
        Stack::new(Vector::new(0), Vector::new(10)),
        true,
    );
    container::test_container(
        Stack::new(Vector::new(10), Vector::new(0)),
        true,
    );
}

#[test]
fn stack_container_test_small() {
    container::test_container(
        Stack::new(Vector::new(10), Vector::new(100)),
        true,
    );
}

use cache::container::{Stack, Vector};
mod container;
mod get;

#[test]
fn container_test() {
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
    container::test_container(
        Stack::new(Vector::new(10), Vector::new(100)),
        true,
    );
}

#[test]
fn get_test() {
    get::test_get(Stack::new(Vector::new(10), Vector::new(100)));
}

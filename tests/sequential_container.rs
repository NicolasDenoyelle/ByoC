mod concurrent;
mod container;

use cache::container::{Sequential, Vector};

#[test]
fn container_test_0() {
    container::test_container(Sequential::new(Vector::new(0)), true);
}

#[test]
fn container_test_small() {
    container::test_container(Sequential::new(Vector::new(100)), true);
}

#[test]
fn concurrent_test_0() {
    concurrent::test_concurrent(Sequential::new(Vector::new(0)), 64);
}

#[test]
fn concurrent_test_small() {
    concurrent::test_concurrent(Sequential::new(Vector::new(100)), 64);
}

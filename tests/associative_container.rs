mod concurrent;
mod container;
mod get;

use cache::container::{Associative, Vector};
use std::collections::hash_map::DefaultHasher;

#[test]
fn container_test() {
    container::test_container(
        Associative::new(
            5,
            10,
            |n| Vector::<u16, u32>::new(n),
            DefaultHasher::new(),
        ),
        false,
    );
}

#[test]
fn get_test() {
    get::test_get(Associative::new(
        5,
        10,
        |n| Vector::<u16, u32>::new(n),
        DefaultHasher::new(),
    ));
}

#[test]
fn concurrent_test() {
    concurrent::test_concurrent(
        Associative::new(30, 30, |n| Vector::new(n), DefaultHasher::new()),
        64,
    );
}

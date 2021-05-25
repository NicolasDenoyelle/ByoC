mod concurrent;
mod container;

use cache::container::{Associative, Vector};
use std::collections::hash_map::DefaultHasher;

#[test]
fn container_test_small() {
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
fn associative_concurrent_test_small() {
    concurrent::test_concurrent(
        Associative::new(30, 30, |n| Vector::new(n), DefaultHasher::new()),
        64,
    );
}

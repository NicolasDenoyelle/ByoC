mod concurrent;
mod container;

use cache::container::{Associative, Vector};
use std::collections::hash_map::DefaultHasher;

#[test]
fn associative_container_test_0() {
    container::test_container(Associative::new(
        0,
        0,
        |n| Vector::<(u16, u32)>::new(n),
        DefaultHasher::new(),
    ));
}

#[test]
fn associative_container_test_small() {
    container::test_container(Associative::new(
        5,
        10,
        |n| Vector::<(u16, u32)>::new(n),
        DefaultHasher::new(),
    ));
}

#[test]
fn associative_concurrent_test_0() {
    concurrent::test_concurrent(
        Associative::new(
            0,
            0,
            |n| Vector::<(u16, u32)>::new(n),
            DefaultHasher::new(),
        ),
        concurrent::rand::range_set(100),
        64,
    );
    concurrent::test_concurrent(
        Associative::new(
            0,
            0,
            |n| Vector::<(u16, u32)>::new(n),
            DefaultHasher::new(),
        ),
        concurrent::rand::rand_set(100),
        64,
    );
}

#[test]
fn associative_concurrent_test_small() {
    concurrent::test_concurrent(
        Associative::new(30, 30, |n| Vector::new(n), DefaultHasher::new()),
        concurrent::rand::range_set(100),
        64,
    );
    concurrent::test_concurrent(
        Associative::new(30, 30, |n| Vector::new(n), DefaultHasher::new()),
        concurrent::rand::rand_set(100),
        64,
    );
}

mod concurrent;
mod container;

use cache::container::concurrent::Sequential;
use cache::container::sequential::Vector;

#[test]
fn sequential_container_test_0() {
    container::test_container(Sequential::new(Vector::new(0)));
}

#[test]
fn sequential_container_test_small() {
    container::test_container(Sequential::new(Vector::new(10)));
}

#[test]
fn sequential_concurrent_test_0() {
    concurrent::test_concurrent(
        Sequential::new(Vector::new(0)),
        concurrent::rand::rand_set(100),
        64,
    );

    concurrent::test_concurrent(
        Sequential::new(Vector::new(0)),
        concurrent::rand::range_set(100),
        64,
    );
}

#[test]
fn sequential_concurrent_test_small() {
    concurrent::test_concurrent(
        Sequential::new(Vector::new(100)),
        concurrent::rand::rand_set(1000),
        64,
    );

    concurrent::test_concurrent(
        Sequential::new(Vector::new(100)),
        concurrent::rand::range_set(1000),
        64,
    );
}

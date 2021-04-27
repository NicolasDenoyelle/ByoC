mod container;
// mod packed;
// mod sequential;
use cache::container::Vector;

#[test]
fn container_test_0() {
    container::test_container(Vector::new(0));
}

#[test]
fn container_test_100() {
    container::test_container(Vector::new(100));
}

// #[test]
// fn packed_container_test_0() {
//     packed::test_container(Vector::new(0));
// }

// #[test]
// fn packed_container_test_small() {
//     packed::test_container(Vector::new(100));
// }

// #[test]
// fn vector_sequential_test_0() {
//     sequential::test_sequential(Vector::new(0));
// }

// #[test]
// fn vector_sequential_test_small() {
//     sequential::test_sequential(Vector::new(100));
// }

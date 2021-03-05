use cache::container::sequential::{TopK, Vector};
mod container;
mod sequential;

#[test]
fn topk_container_test_0() {
    container::test_container(TopK::new(Vector::new(0)));
}

#[test]
fn topk_container_test_small() {
    container::test_container(TopK::new(Vector::new(10)));
}

#[test]
fn topk_container_test_large() {
    container::test_container(TopK::new(Vector::new(1000)));
}

#[test]
fn topk_sequential_test_0() {
    sequential::test_sequential(TopK::new(Vector::new(0)));
}

#[test]
fn topk_sequential_test_small() {
    sequential::test_sequential(TopK::new(Vector::new(100)));
}

use cache::container::{TopK, Vector};
mod packed;
mod sequential;

#[test]
fn topk_container_test_0() {
    packed::test_container(TopK::new(Vector::new(0)));
}

#[test]
fn topk_container_test_small() {
    packed::test_container(TopK::new(Vector::new(100)));
}

#[test]
fn topk_sequential_test_0() {
    sequential::test_sequential(TopK::new(Vector::new(0)));
}

#[test]
fn topk_sequential_test_small() {
    sequential::test_sequential(TopK::new(Vector::new(100)));
}

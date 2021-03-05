use cache::container::sequential::Map;
mod container;
mod sequential;

#[test]
fn map_container_test_0() {
    container::test_container(Map::new(0));
}

#[test]
fn map_container_test_small() {
    container::test_container(Map::new(10));
}

#[test]
fn map_container_test_large() {
    container::test_container(Map::new(1000));
}

#[test]
fn map_sequential_test_0() {
    sequential::test_sequential(Map::new(0));
}

#[test]
fn map_sequential_test_small() {
    sequential::test_sequential(Map::new(100));
}

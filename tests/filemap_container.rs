mod packed;
use cache::container::sequential::FileMap;

#[test]
fn filemap_container_test_0() {
    packed::test_container(unsafe {
        FileMap::new("filemap_container_test_0", 0, false).unwrap()
    });
}

#[test]
fn filemap_container_test_small() {
    packed::test_container(unsafe {
        FileMap::new("filemap_container_test_small", 10, false).unwrap()
    });
}

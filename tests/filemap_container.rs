mod packed;
use cache::container::FileMap;

#[test]
fn filemap_container_test_0() {
    packed::test_container(unsafe {
        FileMap::new::<u16, u32>(
            "filemap_container_test_0",
            0,
            false,
            1024,
        )
        .unwrap()
    });
}

#[test]
fn filemap_container_test_small() {
    packed::test_container(unsafe {
        FileMap::new::<u16, u32>(
            "filemap_container_test_small",
            10,
            false,
            1024,
        )
        .unwrap()
    });
}

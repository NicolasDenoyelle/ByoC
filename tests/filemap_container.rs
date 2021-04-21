mod packed;
use cache::container::FileMap;
#[test]
fn filemap_container_test_0() {
    let container = unsafe {
        FileMap::new::<u16, u32>(
            "filemap_container_test_0",
            0,
            false,
            1024,
        )
        .unwrap()
    };

    std::panic::set_hook(Box::new(|_| {
        #[allow(unused_must_use)]
        {
            std::fs::remove_file("filemap_container_test_0");
        }
    }));

    packed::test_container(container);
}

#[test]
fn filemap_container_test_small() {
    let container = unsafe {
        FileMap::new::<u16, u32>(
            "filemap_container_test_small",
            10,
            false,
            1024,
        )
        .unwrap()
    };

    std::panic::set_hook(Box::new(|_| {
        #[allow(unused_must_use)]
        {
            std::fs::remove_file("filemap_container_test_small");
        }
    }));
    packed::test_container(container);
}

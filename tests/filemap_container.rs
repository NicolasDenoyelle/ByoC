mod packed;
// mod sequential;
use cache::container::sequential::FileMap;

#[test]
fn filemap_container_test_0() {
    packed::test_container(
        FileMap::new("filemap_container_test_0", 0, false).unwrap(),
    );
}

// #[test]
// fn filemap_container_test_small() {
//     packed::test_container(
//         FileMap::new("filemap_container_test_small", 100, false).unwrap(),
//     );
// }

// #[test]
// fn vector_container_test_large() {
//     packed::test_container(FileMap::new(1000));
// }

// #[test]
// fn vector_sequential_test_0() {
//     sequential::test_sequential(FileMap::new(0));
// }

// #[test]
// fn vector_sequential_test_small() {
//     sequential::test_sequential(FileMap::new(100));
// }

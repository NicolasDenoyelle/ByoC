#[cfg(feature = "filemap")]
use std::path::Path;
#[cfg(feature = "filemap")]
use tempfile::NamedTempFile;
#[cfg(feature = "filemap")]
mod packed;
#[cfg(feature = "filemap")]
use cache::container::FileMap;

#[cfg(feature = "filemap")]
#[test]
fn filemap_container_test_0() {
    let tmp_path = NamedTempFile::new().unwrap().into_temp_path();
    let tmp_string =
        String::from(AsRef::<Path>::as_ref(&tmp_path).to_str().unwrap());

    let container =
        FileMap::<(u16, u32)>::new(&tmp_string, 0, false, 1024).unwrap();
    packed::test_container(container);
}

#[cfg(feature = "filemap")]
#[test]
fn filemap_container_test_small() {
    let tmp_path = NamedTempFile::new().unwrap().into_temp_path();
    let tmp_string =
        String::from(AsRef::<Path>::as_ref(&tmp_path).to_str().unwrap());

    let container =
        FileMap::<(u16, u32)>::new(&tmp_string, 10, false, 1024).unwrap();
    packed::test_container(container);
}

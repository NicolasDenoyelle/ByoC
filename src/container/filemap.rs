use crate::container::Container;
use crate::marker::Packed;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    cmp::{Eq, Ordering},
    fs::{remove_file, File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut, Drop},
    path::{Path, PathBuf},
};
use tempfile::{NamedTempFile, TempPath};

/// Structure with specified contiguous memory layout
/// representing an element key / value.
/// The struct is an option specifying weather this
/// element is a valid initialized element or a hole.
struct FileMapElement {}

impl FileMapElement {
    /// Read a `stream` and retrieve consecutive key/value `(K,V)` pairs
    /// using intermediate `buffer` to store read elements. When elements
    /// are tagged as unset, None is returned instead of a key/value pair.
    pub fn read<F: Read + Seek, T: DeserializeOwned>(
        stream: &mut F,
    ) -> Result<(u64, Option<T>), ()> {
        let pos = stream.seek(SeekFrom::Current(0)).unwrap();
        match bincode::deserialize_from::<&mut F, bool>(stream) {
            Err(_) => {
                stream.seek(SeekFrom::Start(pos)).unwrap();
                Err(())
            }
            Ok(set) => {
                match bincode::deserialize_from::<&mut F, T>(stream) {
                    Err(_) => {
                        stream.seek(SeekFrom::Start(pos)).unwrap();
                        Err(())
                    }
                    Ok(v) => {
                        if !set {
                            Ok((pos, None))
                        } else {
                            Ok((pos, Some(v)))
                        }
                    }
                }
            }
        }
    }

    /// Write an element to `stream`.
    pub fn write<'a, F: Seek + Write, T: Serialize>(
        stream: &mut F,
        value: &'a T,
    ) -> Result<(), ()> {
        let pos = stream.seek(SeekFrom::Current(0)).unwrap();
        let set = true;
        match bincode::serialize_into::<&mut F, bool>(stream, &set) {
            Err(_) => {
                stream.seek(SeekFrom::Start(pos)).unwrap();
                Err(())
            }
            Ok(_) => {
                match bincode::serialize_into::<&mut F, &'a T>(
                    stream, &value,
                ) {
                    Err(_) => {
                        stream.seek(SeekFrom::Start(pos)).unwrap();
                        Err(())
                    }
                    Ok(_) => Ok(()),
                }
            }
        }
    }

    /// Tag next element in `stream` as not set.
    /// On success, stream is forwarded by the size of one element.
    pub fn unset<
        F: Read + Seek + Write,
        T: Serialize + DeserializeOwned,
    >(
        stream: &mut F,
    ) -> Result<(), ()> {
        let pos = stream.seek(SeekFrom::Current(0)).unwrap();
        let set = false;
        match bincode::serialize_into::<&mut F, bool>(stream, &set) {
            Err(_) => {
                stream.seek(SeekFrom::Start(pos)).unwrap();
                Err(())
            }
            Ok(_) => Ok(()),
        }
    }
}

//------------------------------------------------------------------------//
// Iterator over a file.
//------------------------------------------------------------------------//

enum FileMapIteratorPath {
    TmpPath(TempPath),
    PhantomPath,
}

/// Iterator over a file containing consecutive elements.
/// This iterator returns a tuple where first element is the offset
/// of the item in file and second element is an optional value that
/// has been read from the file.
/// The file may contain holes (unset elements) in which case
/// the second element is None. This iterator can buffer reads if provided
/// stream is buffered.
pub struct FileMapIterator<T, F>
where
    T: DeserializeOwned,
    F: Read + Seek,
{
    file: F,
    // When dropped, the temp file is deleted.
    // This is used mainly to flush the container in a temporary
    // file.
    #[allow(dead_code)]
    path: FileMapIteratorPath,
    unused_t: PhantomData<T>,
}

impl<T, F> FileMapIterator<T, F>
where
    T: DeserializeOwned,
    F: Read + Seek,
{
    fn new(file: F, path: FileMapIteratorPath) -> Self {
        FileMapIterator {
            file: file,
            path: path,
            unused_t: PhantomData,
        }
    }
}

impl<T, F> Iterator for FileMapIterator<T, F>
where
    T: DeserializeOwned + Serialize,
    F: Read + Seek,
{
    type Item = (u64, Option<T>);

    fn next(&mut self) -> Option<Self::Item> {
        match FileMapElement::read::<_, T>(&mut self.file) {
            Err(_) => None,
            Ok(x) => Some(x),
        }
    }
}

//------------------------------------------------------------------------//
//  Iterator to take elements out
//------------------------------------------------------------------------//

pub struct FileMapTakeIterator<'a, K, V, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    F: Read + Write + Seek,
{
    file: F,
    key: &'a K,
    unused_v: PhantomData<V>,
}

impl<'a, K, V, F> Iterator for FileMapTakeIterator<'a, K, V, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    F: Read + Write + Seek,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match FileMapElement::read::<_, (K, V)>(&mut self.file) {
                Err(_) => break None,
                Ok((off, opt)) => match opt {
                    None => {}
                    Some((k, v)) => {
                        if &k == self.key {
                            self.file.seek(SeekFrom::Start(off)).unwrap();
                            FileMapElement::unset::<_, (K, V)>(
                                &mut self.file,
                            )
                            .unwrap();
                            break Some((k, v));
                        }
                    }
                },
            }
        }
    }
}

//------------------------------------------------------------------------//
// Elements for get method.
//------------------------------------------------------------------------//

/// Struct returned from calling [`get()`](struct.FileMap.html#tymethod.get)
/// method with a [`FileMap`](struct.FileMap.html) container.
///
/// `FileMapValue` struct implements `Deref` and `DerefMut` traits to
/// access reference to the value it wraps.
/// The value it wraps originate from a [`FileMap`](struct.FileMap.html)
/// container. This value may be a cache
/// [reference](../reference/trait.Reference.html). References implement
/// interior mutability such that when they are dereferenced to access their
/// inner value, they can update their metadata about accesses.
/// Hence, values wrapped in this struct are expected to be updated.
/// Therefore, they need to be written back to the file when they cease
/// to be used to commit their metadata update.
/// As a consequence, when this structure is dropped, it is written back
/// to the [`FileMap`](struct.FileMap.html) it was taken from.
pub struct FileMapValue<'a, T>
where
    T: 'a + Serialize,
{
    file: File,
    offset: u64,
    value: T,
    unused_lifetime: PhantomData<&'a T>,
}

impl<'a, T> FileMapValue<'a, T>
where
    T: 'a + Serialize,
{
    fn new(file_handle: &File, offset: u64, value: T) -> Self {
        FileMapValue {
            file: file_handle.try_clone().unwrap(),
            offset: offset,
            value: value,
            unused_lifetime: PhantomData,
        }
    }
}

impl<'a, K, V> Deref for FileMapValue<'a, (K, V)>
where
    K: 'a + Eq + Serialize,
    V: 'a + Ord + Serialize,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value.1
    }
}

impl<'a, K, V> DerefMut for FileMapValue<'a, (K, V)>
where
    K: 'a + Eq + Serialize,
    V: 'a + Ord + Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value.1
    }
}

impl<'a, T> Drop for FileMapValue<'a, T>
where
    T: 'a + Serialize,
{
    fn drop(&mut self) {
        self.file.seek(SeekFrom::Start(self.offset)).unwrap();
        FileMapElement::write(&mut self.file, &self.value).unwrap();
    }
}

//------------------------------------------------------------------------//
// Container Filemap
//------------------------------------------------------------------------//

/// A [`Container`](trait.Container.html) implementation for key/value
/// elements stored into a file.
///
/// [`FileMap`](struct.FileMap.html) container is a file where elements
/// are stored contiguously. Elements stored inside a
/// [`FileMap`](struct.FileMap.html) are assumed to
/// [`Serialize`](../../serde/trait.Serialize.html) to elements of the
/// [same binary size](../../bincode/fn.serialized_size.html).  
/// When an element is taken out of the container, it leaves a hole that
/// may be filled on future insertion of a non existing key.  
/// The container has a small memory footprint, since the bulk of it is
/// stored in a file.
/// While using a [`FileMap`](struct.FileMap.html) container,
/// temporary additional buffer size is created to buffer IO read
/// operations.  
/// This container is intended to be optimized by combining it with:
/// in-memory cache multiple files/sets in concurrent associative container,
/// optimized replacement policy, and so on...  
/// [`FileMap`](../struct.FileMap.html) container implements the marker
/// trait [`Packed`](../marker/trait.Packed.html).
/// Therefore, it will accept new as long as it is not full.
/// It also implements a [`get()`](struct.FileMap.html#tymethod.get)
/// method that returns values wrapped into a smart
/// pointer. When the smart pointer goes out of scope, the value is written
/// back to the file to update values possibly wrapped into a
/// [`Reference`](../reference/trait.Reference.html) with interior
/// mutability.
///
/// ## Example:
/// ```
/// use cache::container::{Container, FileMap};
///
/// let mut container = FileMap::new("example_filemap", 2, false, 1024).unwrap();
///
/// // If test fails, delete created file because destructor is not called.
/// std::panic::set_hook(Box::new(|_| {
///   #[allow(unused_must_use)]
///   {
///     std::fs::remove_file("example_filemap");
///   }
/// }));
///
/// assert!(container.push(0u32, 0u32).is_none());
/// assert!(container.push(1u32, 1u32).is_none());
/// assert!(container.push(2u32, 2u32).unwrap() == (1u32,1u32));
/// ```
pub struct FileMap<T: Serialize + DeserializeOwned> {
    file: File,
    path: PathBuf,
    persistant: bool,
    capacity: usize,
    buffer_size: usize,
    serialized_size: u64,
    unused_t: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Drop for FileMap<T> {
    fn drop(&mut self) {
        if !self.persistant {
            remove_file(&self.path).unwrap();
        }
    }
}

impl<K, V> FileMap<(K, V)>
where
    K: Eq + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    /// Instanciate a new [`FileMap`](struct.FileMap.html) with a maximum
    /// of `capacity` keys, stored with their value in the file
    /// named `filename`. If `persistant` is `true`, the inner file will
    /// not be deleted when the container is dropped. When walking the file
    /// to perform container operations, [`FileMap`](struct.FileMap.html) will
    /// use `buffer_size` bytes of space to buffer IO operations.
    pub fn new(
        filename: &str,
        capacity: usize,
        persistant: bool,
        buffer_size: usize,
    ) -> Result<Self, std::io::Error> {
        let pb = PathBuf::from(filename);
        let file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&pb)
        {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        Ok(FileMap {
            file: file.try_clone().unwrap(),
            path: pb,
            capacity: capacity,
            persistant: persistant,
            buffer_size: buffer_size,
            serialized_size: 0u64,
            unused_t: PhantomData,
        })
    }

    pub fn get<'c>(
        &'c mut self,
        key: &'c K,
    ) -> Box<dyn Iterator<Item = FileMapValue<'c, (K, V)>> + 'c> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        Box::new(
            FileMapIterator::<(K, V), _>::new(
                BufReader::with_capacity(
                    self.buffer_size,
                    self.file.try_clone().unwrap(),
                ),
                FileMapIteratorPath::PhantomPath,
            )
            .filter_map(move |(off, opt)| match opt {
                None => None,
                Some((k, v)) => {
                    if &k == key {
                        Some(FileMapValue::new(&self.file, off, (k, v)))
                    } else {
                        None
                    }
                }
            }),
        )
    }

    /// Get an `Iterator` over a clone of the
    /// [`FileMap`](../struct.FileMap.html) file to read its elements.
    /// Elements returned by the iterator are copies of elements in the file
    /// and are never written back to the file if modified.
    /// Elements returned by the iterator are tuple where first element
    /// is the offset of element in file and second element is an `Option`
    /// over element's value which is either `None` if the iteration encountered
    /// a hole, else the value at that offset.
    fn iter_owned(&self) -> FileMapIterator<(K, V), BufReader<File>> {
        let mut f = self.file.try_clone().unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::new(
            BufReader::with_capacity(self.buffer_size, f),
            FileMapIteratorPath::PhantomPath,
        )
    }

    /// Write an element at desired `offset`.
    /// This function is only used internally to factorize code.
    fn write(&mut self, offset: SeekFrom, t: &(K, V)) {
        let serialized_size = bincode::serialized_size(t).unwrap();
        if self.serialized_size == 0 {
            self.serialized_size = serialized_size;
        }
        if self.serialized_size != serialized_size {
            panic!("Trying to write elements with different sizes in FileMap container {:?}", self.path);
        } else {
            self.file.seek(offset).unwrap();
            FileMapElement::write(&mut self.file, t).unwrap();
        }
    }
}

//------------------------------------------------------------------------//
// Container impl
//------------------------------------------------------------------------//

impl<'a, K, V> Container<'a, K, V> for FileMap<(K, V)>
where
    K: 'a + Eq + DeserializeOwned + Serialize,
    V: 'a + Ord + DeserializeOwned + Serialize,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        self.iter_owned().filter(|(_, x)| x.is_some()).count()
    }

    fn contains(&self, key: &K) -> bool {
        self.iter_owned()
            .any(|(_, e): (u64, Option<(K, V)>)| match e {
                None => false,
                Some((k, _)) => &k == key,
            })
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        // Create temporary file path
        let tmp_path = NamedTempFile::new().unwrap().into_temp_path();
        // Move container file to temporary file.
        std::fs::rename(&self.path, AsRef::<Path>::as_ref(&tmp_path))
            .unwrap();

        // Create empty file for container.
        self.file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&self.path)
            .unwrap();

        // Open temporary file of flush iterator.
        let tmp_string = String::from(
            AsRef::<Path>::as_ref(&tmp_path).to_str().unwrap(),
        );
        let file = OpenOptions::new().read(true).open(tmp_string).unwrap();

        // Map temporary file iterator to return only set valid items.
        Box::new(
            FileMapIterator::<(K, V), _>::new(
                BufReader::with_capacity(self.buffer_size, file),
                FileMapIteratorPath::TmpPath(tmp_path),
            )
            .filter_map(|(_, e)| e),
        )
    }

    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
        let mut f = self.file.try_clone().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        Box::new(FileMapTakeIterator {
            file: f,
            key: key,
            unused_v: PhantomData,
        })
    }

    fn clear(&mut self) {
        self.file.set_len(0).unwrap()
    }

    fn pop(&mut self) -> Option<(K, V)> {
        let victim = self.iter_owned().max_by(
            |(_, o1): &(u64, Option<(K, V)>),
             (_, o2): &(u64, Option<(K, V)>)| match (o1, o2) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some((_, v1)), Some((_, v2))) => (&v1).cmp(&v2),
            },
        );

        match victim {
            None => None,
            Some((_, None)) => None,
            Some((off, Some((k, v)))) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMapElement::unset::<_, (K, V)>(&mut self.file)
                    .unwrap();
                Some((k, v))
            }
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        // Find a victim to evict: the minimum element.
        let mut victim: Option<(u64, (K, V))> = None;
        // If there are holes then we insert in a hole.
        let mut spot: Option<u64> = None;
        // Count number of elements. If container is not full and their is
        // no hole, we append at the end.
        let mut n_elements = 0usize;

        // We start walking the file in search for holes and potential
        // victims.
        for (off, opt) in self.iter_owned() {
            n_elements += 1;
            match opt {
                None => {
                    spot = Some(off);
                    break;
                }
                Some((k, v)) => {
                    victim = match victim {
                        None => Some((off, (k, v))),
                        Some((off1, (k1, v1))) => {
                            if v > v1 {
                                Some((off, (k, v)))
                            } else {
                                Some((off1, (k1, v1)))
                            }
                        }
                    }
                }
            }
        }

        match spot {
            Some(off) => {
                self.write(SeekFrom::Start(off), &(key, reference));
                None
            }
            None => {
                if n_elements < self.capacity {
                    self.write(SeekFrom::End(0), &(key, reference));
                    None
                } else {
                    match victim {
                        None => Some((key, reference)),
                        Some((off, x)) => {
                            self.write(
                                SeekFrom::Start(off),
                                &(key, reference),
                            );
                            Some(x)
                        }
                    }
                }
            }
        }
    }
}

impl<'a, K, V> Packed<'a, K, V> for FileMap<(K, V)>
where
    K: 'a + Eq + DeserializeOwned + Serialize,
    V: 'a + Ord + DeserializeOwned + Serialize,
{
}

//------------------------------------------------------------------------//
// Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::{FileMap, FileMapElement};
    use crate::container::Container;
    use std::io::{Seek, SeekFrom, Write};
    use std::path::Path;
    use tempfile::{tempfile, NamedTempFile};

    #[test]
    fn test_filemap_element() {
        let mut file = tempfile().unwrap();

        // Write elements to file.
        let input: Vec<(usize, usize)> =
            (0usize..16usize).map(|i| (i, i)).collect();
        for (k, v) in input.iter() {
            FileMapElement::write(&mut file, &(*k, *v)).unwrap();
        }
        file.flush().unwrap();

        // Read elements from file.
        (&file).seek(SeekFrom::Start(0)).unwrap();
        let mut output: Vec<(usize, usize)> = Vec::new();
        loop {
            match FileMapElement::read(&mut file) {
                Err(_) => break,
                Ok((_, e)) => output.push(e.unwrap()),
            };
        }
    }

    #[test]
    fn test_filemap() {
        let tmp_path = NamedTempFile::new().unwrap().into_temp_path();
        let tmp_string = String::from(
            AsRef::<Path>::as_ref(&tmp_path).to_str().unwrap(),
        );

        let mut fm = FileMap::new(&tmp_string, 10, false, 1024).unwrap();

        // Push test
        for i in (0usize..10usize).rev() {
            assert!(fm.push(i, i).is_none());
        }

        // Contain test
        for i in (0usize..10usize).rev() {
            assert!(fm.contains(&i));
        }

        // Pop test
        assert_eq!(fm.pop().unwrap().0, 9usize);
        let i = 9usize;
        assert!(!fm.contains(&i));

        // Test pop on push when full.
        assert!(fm.push(9usize, 9usize).is_none());
        match fm.push(11usize, 11usize) {
            None => panic!("Full filemap not popping."),
            Some((k, _)) => {
                assert_eq!(k, 9usize);
            }
        }

        // Test pop on push of an existing key.
        match fm.push(4usize, 4usize) {
            None => panic!("Full filemap not popping."),
            Some((k, _)) => {
                assert_eq!(k, 11usize);
            }
        }

        // Test empty container.
        assert_eq!(fm.pop().unwrap().0, 8usize);
        for i in vec![7, 6, 5, 4, 4, 3, 2, 1, 0].iter() {
            assert_eq!(fm.pop().unwrap().0, *i as usize);
        }
        assert!(fm.pop().is_none());
        assert_eq!(fm.count(), 0);
    }
}

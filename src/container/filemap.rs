use crate::container::{Container, Get};
use crate::marker::Packed;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cmp::{Eq, Ordering},
    fmt::Debug,
    fs::{remove_file, File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut, Drop},
    path::{Path, PathBuf},
};
use tempfile::{NamedTempFile, TempPath};

/// Structure with specified contiguous memory layout
/// representing an element key / value.
/// The struct also contains a boolean specifying weather this
/// element is a valid initialized element or not.
#[derive(Serialize, Deserialize, Debug)]
struct FileMapElement<'a, K, V>
where
    K: 'a + Serialize + DeserializeOwned + Debug,
    V: 'a + Serialize + DeserializeOwned + Debug,
{
    set: bool,
    key: K,
    value: V,
    unused_lifetime: PhantomData<&'a K>,
}

impl<'a, K, V> FileMapElement<'a, K, V>
where
    K: 'a + Serialize + DeserializeOwned + Debug,
    V: 'a + Serialize + DeserializeOwned + Debug,
{
    /// Read a `stream` and retrieve consecutive key/value `(K,V)` pairs
    /// using intermediate `buffer` to store read elements. When elements
    /// are tagged as unset, None is pushed into returned `Vec` instead of
    /// a key/value pair.
    ///
    /// Safety:
    /// This method is safe only if `stream` has been exclusively written
    /// with the same `FileMapElement<K,V>` type and binary representation.
    /// The stream cursor must also be pointing at the beginning of next element
    /// or stream end.
    pub fn read<F: Read + Seek>(
        stream: &mut F,
    ) -> Result<(u64, Option<(K, V)>), ()> {
        let pos = stream.seek(SeekFrom::Current(0)).unwrap();
        match bincode::deserialize_from::<&mut F, Self>(stream) {
            Err(e) => {
                stream.seek(SeekFrom::Start(pos)).unwrap();
                Err(())
            }
            Ok(e) => {
                if e.set {
                    Ok((pos, Some((e.key, e.value))))
                } else {
                    Ok((pos, None))
                }
            }
        }
    }

    /// Write a key/value `(K,V)` pair wrapped into a
    /// `FileMapElement<(K,V)>` to `stream`.
    pub fn write<F: Write>(
        stream: &mut F,
        key: &K,
        value: &V,
    ) -> Result<(), ()> {
        let e = unsafe {
            let mut e = std::mem::MaybeUninit::<Self>::uninit();
            (*(e.as_mut_ptr())).set = true;
            std::ptr::copy_nonoverlapping(
                key,
                &mut (*(e.as_mut_ptr())).key as *mut K,
                std::mem::size_of::<K>(),
            );
            std::ptr::copy_nonoverlapping(
                value,
                &mut (*(e.as_mut_ptr())).value as *mut V,
                std::mem::size_of::<V>(),
            );
            e.assume_init()
        };

        match bincode::serialize_into::<&mut F, Self>(stream, &e) {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
    }

    /// Tag next element in `stream` as not set.
    /// On success, stream is forwarded by the size of one element.
    pub fn unset<F: Write>(stream: &mut F) -> Result<(), ()> {
        // SAFETY: We write exactly one element with flag `set` set
        // to false. When this element get read later on, its other
        // fields will not be accesed due to this flag. Therefore
        // No need to initialize them.
        let e = {
            let mut e = std::mem::MaybeUninit::<Self>::uninit();
            (*e.as_mut_ptr()).set = false;
            e.assume_init()
        };
        match bincode::serialize_into::<&mut F, Self>(stream, &e) {
            Err(_) => Err(()),
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

/// Iterator over a file containing consecutive `FileMapElement<K,V>`.
/// This iterator returns a tuple where first element is the offset
/// of the item in file and second element is a `FileMapElement<K,V>`
/// `Option`. The file may contain holes (unset elements) in which case
/// the second element is None. This iterator buffers file read in
/// an internal `bytes` buffer to land file reads and a `buffer` iterator
/// containing elements read in `bytes`.
struct FileMapIterator<'a, K, V, F>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
    F: Read + Seek,
{
    file: F,
    // When dropped, the temp file is deleted.
    // This is used mainly to flush the container in a temporary
    // file.
    #[allow(dead_code)]
    path: FileMapIteratorPath,
    unused_k: PhantomData<&'a K>,
    unused_v: PhantomData<&'a V>,
}

impl<'a, K, V, F> FileMapIterator<'a, K, V, F>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
    F: Read + Seek,
{
    fn new(
        file: F,
        path: FileMapIteratorPath,
        buffer_size: usize,
    ) -> Self {
        FileMapIterator {
            file: file,
            path: path,
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }
}

impl<'a, K, V, F> Iterator for FileMapIterator<'a, K, V, F>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
    F: Read + Seek,
{
    type Item = (u64, Option<(K, V)>);

    fn next(&mut self) -> Option<Self::Item> {
        match FileMapElement::<K, V>::read::<F>(&mut self.file) {
            Err(_) => None,
            Ok(x) => Some(x),
        }
    }
}

/// A [`Container`](../trait.Container.html) for key value store with a
/// maximum size stored into a file.
///
/// The container has small memory footprint, since the bulk of it is stored
/// in a file. Eventhough IO reads operation are buffered, nearly
/// all [`Container`](../trait.Container.html) methods will
/// require to read the entire file. The file where this container
/// is mapped contains only consecutive elements that may be unset and
/// leave a whole in the file for insertions. This is intended to limit
/// file growth and improve performance since the file will be read entirely
/// once on almost all operation.
/// Elements are stored to fit any hole in the file so that the
/// container does not pop until the file is filled with elements.
/// This container is intended to be optimized by combining it with:
/// in-memory cache multiple files/sets in concurrent associative container,
/// optimized replacement policy, and so on...
/// This container implements the marker trait `Packed` which means,
/// that it will accept new elements with non existing keys as long
/// as it is not full.
///
/// ## Example:
/// ```
/// use cache::container::{Container, FileMap};
///
/// let mut container = unsafe {
///     FileMap::new::<u32,u32>("example_filemap", 2, false, 1024).unwrap()
/// };
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
pub struct FileMap {
    file: File,
    path: PathBuf,
    persistant: bool,
    capacity: usize,
    buffer_size: usize,
}

impl Drop for FileMap {
    fn drop(&mut self) {
        if !self.persistant {
            remove_file(&self.path).unwrap();
        }
    }
}

impl FileMap {
    /// Instanciate a new [`FileMap`](struct.FileMap.html) with a maximum
    /// of `capacity` keys, stored with their value in the file
    /// named `filename`. If `persistant` is `true`, the inner file will
    /// not be deleted when the container is dropped. When walking the file
    /// to perform container operations, [`FileMap`](struct.FileMap.html) will
    /// use `buffer_size` bytes of space to buffer IO operations.
    ///
    /// SAFETY:
    /// Keys and values must be safely writable and readable in-place, i.e.
    /// they do not contain pointers that would be invalid to read from a
    /// file and they have a fixed size, e.g they are not dynamically
    /// sized strings or vectors. Keys and Values must also have a
    /// consistent struct layout across compilations if the underlying
    /// `FileMap` file is going to be used by in this context. If the file
    /// already exists it must not be corrupted and only contains zero or
    /// several valid or unset consecutive elements.
    pub unsafe fn new<K: Sized, V: Sized>(
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
        })
    }
}

//------------------------------------------------------------------------//
// Container impl
//------------------------------------------------------------------------//

impl<'a, K, V> Container<'a, K, V> for FileMap
where
    K: 'a + Serialize + Deserialize<'a> + Debug + Eq,
    V: 'a + Serialize + Deserialize<'a> + Debug + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        let mut f = self.file.try_clone().unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(self.buffer_size, f),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        )
        .filter(|(_, x)| x.is_some())
        .count()
    }

    fn contains(&self, key: &K) -> bool {
        let mut f = self.file.try_clone().unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(self.buffer_size, f),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        )
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
            FileMapIterator::<'a, K, V, _>::new(
                BufReader::with_capacity(self.buffer_size, file),
                FileMapIteratorPath::TmpPath(tmp_path),
                self.buffer_size,
            )
            .filter_map(|(_, e)| e),
        )
    }

    fn take(&mut self, key: &K) -> Option<V> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        match FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(
                self.buffer_size,
                self.file.try_clone().unwrap(),
            ),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        )
        .find(|(_, e): &(u64, Option<(K, V)>)| match e {
            None => false,
            Some((k, _)) => k == key,
        }) {
            Some((off, Some((_, v)))) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMapElement::<K, V>::unset(&mut self.file).unwrap();
                Some(v)
            }
            _ => None,
        }
    }

    fn clear(&mut self) {
        self.file.set_len(0).unwrap()
    }

    fn pop(&mut self) -> Option<(K, V)> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        let victim = FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(
                self.buffer_size,
                self.file.try_clone().unwrap(),
            ),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        )
        .max_by(
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
                FileMapElement::<K, V>::unset(&mut self.file).unwrap();
                Some((k, v))
            }
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        // Flush any outstanding write because we want to read the whole
        // file.
        self.file.flush().unwrap();
        // Position ourselves at the beginning of the file.
        self.file.seek(SeekFrom::Start(0)).unwrap();

        // Find a victim to evict: Either an element with the same key
        // or the minimum element.
        let mut victim: Option<(u64, (K, V))> = None;
        // If there are holes and the victim does not have the same key
        // Then we insert in a whole.
        let mut spot: Option<u64> = None;
        // Count number of elements.
        let mut n_elements = 0usize;

        // We start walking the file in search for the same key, holes and
        // potential victims.
        // Everything is one in one pass.
        for (off, opt) in FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(
                self.buffer_size,
                self.file.try_clone().unwrap(),
            ),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        ) {
            n_elements += 1;
            match opt {
                None => {
                    spot = Some(off);
                }
                Some((k, v)) => {
                    // evict.
                    if k == key {
                        victim = Some((off, (k, v)));
                        break;
                    } else {
                        victim = match (spot, victim) {
                            // There is a hole, then we don't care about victims.
                            (Some(_), vict) => vict,
                            // There is no current victim and no hole then,
                            // This is the current victim.
                            (None, None) => Some((off, (k, v))),
                            // Next victim is the element with max reference.
                            (None, Some((off1, (k1, v1)))) => {
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
        }

        match (victim, spot) {
            // No victim and no spot... It means the file is empty.
            (None, None) => {
                // If there is room, we append element at the end of the file.
                if n_elements < self.capacity {
                    self.file.seek(SeekFrom::End(0)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    None
                }
                // Else we return input.
                else {
                    Some((key, reference))
                }
            }
            // No victim but a spot, then insert in the spot.
            (None, Some(off)) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMapElement::<K, V>::write(
                    &mut self.file,
                    &key,
                    &reference,
                )
                .unwrap();
                None
            }
            // A victim and a spot! If the victim has the same key then
            // We evict the victim, else we fill the spot
            (Some((off, (k, v))), Some(offset)) => {
                if k == key {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::Start(offset)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    None
                }
            }
            // A victim and no spot.
            // If the container is full, then we replace the victim else
            // we append at the end of the file.
            (Some((off, (k, v))), None) => {
                if k == key {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    Some((k, v))
                } else if n_elements >= self.capacity {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::End(0)).unwrap();
                    FileMapElement::<K, V>::write(
                        &mut self.file,
                        &key,
                        &reference,
                    )
                    .unwrap();
                    None
                }
            }
        }
    }
}

impl<'a, K, V> Packed<'a, K, V> for FileMap
where
    K: 'a + Serialize + Deserialize<'a> + Debug + Eq,
    V: 'a + Serialize + Deserialize<'a> + Debug + Ord,
{
}

//------------------------------------------------------------------------//
// Get trait
//------------------------------------------------------------------------//

/// Struct returned from calling [`get()`](trait.Get.html#tymethod.get)
/// method with a [`FileMap`](struct.FileMap.html) container.
///
/// `FileMapValue` struct implements `Deref` and `DerefMut` traits to
/// access reference to the value it wraps.
/// The value it wraps originate from a [`FileMap`](struct.FileMap.html)
/// container. This value is expected to be a cache
/// [reference](../reference/trait.Reference.html). References implement
/// interior mutability such that when they are dereferenced to access their
/// inner value, they can update their metadata about accesses.
/// Hence, values wrapped in this struct are expected to be updated.
/// Therefore, they need to be written back to the file when they cease
/// to be used to commit their metadata update.
/// As a consequence, when this structure is dropped, it is writes back
/// its content to the FileMap.
pub struct FileMapValue<'a, K, V>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
{
    file: File,
    offset: u64,
    key: K,
    value: V,
    unused_lifetime: PhantomData<&'a K>,
}

impl<'a, K, V> FileMapValue<'a, K, V>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
{
    fn new(file_handle: &File, offset: u64, key: K, value: V) -> Self {
        FileMapValue {
            file: file_handle.try_clone().unwrap(),
            offset: offset,
            key: key,
            value: value,
            unused_lifetime: PhantomData,
        }
    }
}

impl<'a, K, V> Deref for FileMapValue<'a, K, V>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, K, V> DerefMut for FileMapValue<'a, K, V>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'a, K, V> Drop for FileMapValue<'a, K, V>
where
    K: 'a + Serialize + Deserialize<'a> + Debug,
    V: 'a + Serialize + Deserialize<'a> + Debug,
{
    fn drop(&mut self) {
        self.file.seek(SeekFrom::Start(self.offset)).unwrap();
        FileMapElement::<K, V>::write(
            &mut self.file,
            &mut self.key,
            &mut self.value,
        )
        .unwrap();
    }
}

impl<'a, K, V> Get<'a, K, V> for FileMap
where
    K: 'a + Serialize + Deserialize<'a> + Debug + Eq,
    V: 'a + Serialize + Deserialize<'a> + Debug + Ord,
{
    type Item = FileMapValue<'a, K, V>;
    fn get(&'a mut self, key: &K) -> Option<Self::Item> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::<'a, K, V, _>::new(
            BufReader::with_capacity(
                self.buffer_size,
                self.file.try_clone().unwrap(),
            ),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        )
        .find_map(|(off, opt)| match opt {
            None => None,
            Some((k, v)) => {
                if &k == key {
                    Some(FileMapValue::new(&self.file, off, k, v))
                } else {
                    None
                }
            }
        })
    }
}

//------------------------------------------------------------------------//
// Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::{FileMap, FileMapElement};
    use crate::container::Container;
    use std::fs::{remove_file, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};

    #[test]
    fn test_filemap_element() {
        let filename: &str = "test_filemap_element";
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();

        std::panic::set_hook(Box::new(|_| {
            #[allow(unused_must_use)]
            {
                remove_file("test_filemap_element");
            }
        }));

        // Write elements to file.
        let input: Vec<(usize, usize)> =
            (0usize..16usize).map(|i| (i, i)).collect();
        for (k, v) in input.iter() {
            FileMapElement::<usize, usize>::write::<_, bincode>(
                &mut file, k, v,
            )
            .unwrap();
        }
        file.flush().unwrap();

        // Read elements from file.
        (&file).seek(SeekFrom::Start(0)).unwrap();
        let output: Vec<(usize, usize)> = Vec::new();
        loop {
            match FileMapElement::<usize, usize>::read::<_, bincode>(
                &mut file,
            ) {
                Err(e) => break,
                Some((_, e)) => output.push(e.unwrap()),
            };
        }
        assert_eq!(input, output);
        remove_file(filename).unwrap();
    }

    #[test]
    fn test_filemap() {
        let mut fm = unsafe {
            FileMap::new::<usize, usize>("test_filemap", 10, false, 1024)
                .unwrap()
        };

        std::panic::set_hook(Box::new(|_| {
            #[allow(unused_must_use)]
            {
                remove_file("test_filemap");
            }
        }));

        // Push test
        for i in (0usize..10usize).rev() {
            assert!(
                Container::<usize, usize>::push(&mut fm, i, i).is_none()
            );
        }
        // Pop test
        assert_eq!(
            Container::<usize, usize>::pop(&mut fm).unwrap().0,
            9usize
        );
        // Contains test
        for i in 0usize..9usize {
            assert!(Container::<usize, usize>::contains(&mut fm, &i));
        }
        let i = 9usize;
        assert!(!Container::<usize, usize>::contains(&mut fm, &i));

        // Test pop on push when full.
        assert!(Container::<usize, usize>::push(&mut fm, 9usize, 9usize)
            .is_none());
        match Container::<usize, usize>::push(&mut fm, 11usize, 11usize) {
            None => panic!("Full filemap not popping."),
            Some((k, _)) => {
                assert_eq!(k, 9usize);
            }
        }

        // Test pop on push of an existing key.
        match Container::<usize, usize>::push(&mut fm, 4usize, 4usize) {
            None => panic!("Full filemap not popping."),
            Some((k, _)) => {
                assert_eq!(k, 4usize);
            }
        }

        // Test empty container.
        assert_eq!(
            Container::<usize, usize>::pop(&mut fm).unwrap().0,
            11usize
        );
        for i in (0usize..9usize).rev() {
            assert_eq!(
                Container::<usize, usize>::pop(&mut fm).unwrap().0,
                i
            );
        }
        assert!(Container::<usize, usize>::pop(&mut fm).is_none());
        assert_eq!(Container::<usize, usize>::count(&mut fm), 0);
    }
}

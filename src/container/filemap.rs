use crate::container::{Container, Get};
use crate::marker::Packed;
use std::{
    cmp::{Eq, Ordering},
    fs::{remove_file, File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
    mem::{size_of, MaybeUninit},
    ops::{Deref, DerefMut, Drop},
    path::{Path, PathBuf},
    slice,
};
use tempfile::{NamedTempFile, TempPath};

/// Structure with specified contiguous memory layout
/// representing an element key / value.
/// The struct also contains a boolean specifying weather this
/// element is a valid initialized element or not.
#[repr(C, packed)]
struct FileMapElement<K, V>
where
    K: Sized,
    V: Sized,
{
    set: bool,
    key: K,
    value: V,
}

impl<K, V> FileMapElement<K, V>
where
    K: Sized,
    V: Sized,
{
    fn size() -> usize {
        std::mem::size_of::<FileMapElement<K, V>>()
    }

    /// Read a `stream` and retrieve consecutive key/value `(K,V)` pairs
    /// using intermediate `buffer` to store read elements. When elements
    /// are tagged as unset, None is pushed into returned `Vec` instead of
    /// a key/value pair.
    /// This method is safe only if `stream` has been exclusively written
    /// with the same `FileMapElement<K,V>` type and binary representation.
    pub unsafe fn read<F: Read + Seek>(
        stream: &mut F,
        buffer: &mut [u8],
    ) -> Result<Vec<(u64, Option<(K, V)>)>> {
        let pos = stream.seek(SeekFrom::Current(0)).unwrap();
        match stream.read(buffer) {
            Ok(s) => {
                let elements = Self::from_bytes(&buffer[..s]);
                let offsets = (pos..).step_by(Self::size());
                Ok(offsets.zip(elements.into_iter()).collect())
            }
            Err(e) => Err(e),
        }
    }

    /// Read raw bytes, make a list of consecutive `FileMapElements` out
    /// of it and return their key/value pair when their `set` flag is set.
    /// This method assumes that `bytes` contains consecutive raw
    /// initialized FileMapElement<K,V>. If not, using this function is
    /// undefined behaviour.
    unsafe fn from_bytes(bytes: &[u8]) -> Vec<Option<(K, V)>> {
        let mut ret: Vec<Option<(K, V)>> = Vec::new();
        for c in bytes.chunks_exact(Self::size()) {
            let mut e = MaybeUninit::<Self>::uninit();
            slice::from_raw_parts_mut(
                e.as_mut_ptr() as *mut u8,
                size_of::<Self>(),
            )
            .copy_from_slice(c);
            let e = e.assume_init();
            if e.set {
                ret.push(Some((e.key, e.value)))
            } else {
                ret.push(None);
            }
        }
        ret
    }

    /// Write a key/value `(K,V)` pair wrapped into a
    /// `FileMapElement<(K,V)>` to `stream`.
    pub fn write(
        stream: &mut dyn Write,
        key: &K,
        value: &V,
    ) -> Result<usize> {
        let e = unsafe {
            let mut e = MaybeUninit::<Self>::uninit();
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

        // SAFETY: slice representation is safe because self is
        // initialized.
        let s = unsafe {
            slice::from_raw_parts(
                &e as *const _ as *const u8,
                Self::size(),
            )
        };
        stream.write(s)
    }

    // /// Write a key/value `(K,V)` pair wrapped into a
    // /// `FileMapElement<(K,V)>` to `stream`.
    // pub fn write_buf(
    //     stream: &mut dyn Write,
    //     elements: Vec<(K, V)>,
    // ) -> Result<usize> {
    //     let elements: Vec<FileMapElement<K, V>> = elements
    //         .into_iter()
    //         .map(|(k, v)| FileMapElement {
    //             set: true,
    //             key: k,
    //             value: v,
    //         })
    //         .collect();
    //     unsafe {
    //         stream.write(std::slice::from_raw_parts(
    //             elements.as_ptr() as *const u8,
    //             elements.len() * FileMapElement::<K, V>::size(),
    //         ))
    //     }
    // }

    /// Tag next element in `stream` as not set.
    /// On success, stream is forwarded by element size.
    pub fn unset(stream: &mut dyn Write) -> Result<usize> {
        // SAFETY: We write exactly one element with flag `set` set
        // to false. When this element get read later on, its other
        // fields will not be accesed due to this flag. Therefore
        // No need to initialize them.
        unsafe {
            let mut e = MaybeUninit::<Self>::uninit();
            (*e.as_mut_ptr()).set = false;
            let e = e.assume_init();
            let s = slice::from_raw_parts(
                &e as *const _ as *const u8,
                Self::size(),
            );
            stream.write(s)
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

struct FileMapIterator<'a, K: 'a + Sized, V: 'a + Sized> {
    file: File,
    // When dropped, the temp file is deleted.
    #[allow(dead_code)]
    path: FileMapIteratorPath,
    buffer: Box<dyn Iterator<Item = (u64, Option<(K, V)>)> + 'a>,
    bytes: Vec<u8>,
}

impl<'a, K: 'a + Sized, V: 'a + Sized> FileMapIterator<'a, K, V> {
    fn new(
        file: File,
        path: FileMapIteratorPath,
        buffer_size: usize,
    ) -> Self {
        FileMapIterator {
            file: file,
            path: path,
            buffer: Box::new(Vec::new().into_iter()),
            bytes: vec![0u8; buffer_size],
        }
    }
}

impl<'a, K: 'a + Sized, V: 'a + Sized> Iterator
    for FileMapIterator<'a, K, V>
{
    type Item = (u64, Option<(K, V)>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let e = self.buffer.next();
            if e.is_some() {
                break e;
            } else {
                match unsafe {
                    FileMapElement::<K, V>::read(
                        &mut self.file,
                        self.bytes.as_mut_slice(),
                    )
                } {
                    Err(_) => break None,
                    Ok(e) => {
                        if e.len() == 0 {
                            break None;
                        } else {
                            self.buffer = Box::new(e.into_iter());
                        }
                    }
                }
            }
        }
    }
}

/// A [`Container`](../trait.Container.html) for key value store with a
/// maximum size stored into a file.
///
/// The container has tiny memory
/// footprint. However, it is not optimized to limit IO operations.
/// Elements are stored to fit any hole in the file so that the
/// container does not pop until the file is filled with elements.
/// This container is extremely slow, i.e almost all methods will
/// require to cross the entire file.
/// This container is intended to be optimized by combining it with:
/// in-memory cache multiple files/sets in concurrent associative container,
/// optimized replacement policy, and so on...
/// This container implements the marker trait `Packed` which means,
/// that it will accept new elements with non existing keys as long
/// as it is not full.
///
/// ## Generics:
///
/// * `K`: The type of key to use.
/// Keys must have a set size known at compile time and must not contain
/// pointers that would be invalid to read later from a file.
/// Keys must be comparable with `Eq` trait.
/// * `V`: The value type stored.
/// Values must have a set size known at compile time and must not contain
/// pointers that would be invalid to read later from a file.
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
/// assert!(container.push(0, 0).is_none());
/// assert!(container.push(1, 1).is_none());
/// assert!(container.push(2, 2).unwrap() == (1,1));
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
        if self.persistant {
            remove_file(&self.path).unwrap();
        }
    }
}

impl FileMap {
    /// Instanciate a new [`FileMap`](struct.FileMap.html) with a maximum
    /// of `capacity` keys, stored with their value in the file
    /// named `filename`. If `persistant` is `true`, the inner file will
    /// not be deleted when the container is dropped.
    ///
    /// SAFETY:
    /// Keys and values must be safely writable and readable in-place, i.e.
    /// they do not contain pointers that would be invalid to read from a
    /// file and they have a fixed size, e.g they are not dynamically
    /// sized strings or vectors. Keys and Values must also have a
    /// consistent struct layout across compilations if the underlying
    /// `FileMap` file is going to be used by in this context.
    pub unsafe fn new<K, V>(
        filename: &str,
        capacity: usize,
        persistant: bool,
        buffer_size: usize,
    ) -> Result<Self> {
        let buffer_size =
            std::cmp::min(buffer_size, FileMapElement::<K, V>::size());
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
    K: 'a + Sized + Eq,
    V: 'a + Sized + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        let mut f = self.file.try_clone().unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::<'a, K, V>::new(
            f,
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

        FileMapIterator::<'a, K, V>::new(
            f,
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
            FileMapIterator::<'a, K, V>::new(
                file,
                FileMapIteratorPath::TmpPath(tmp_path),
                self.buffer_size,
            )
            .filter_map(|(_, e)| e),
        )
    }

    fn take(&mut self, key: &K) -> Option<V> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        match FileMapIterator::<'a, K, V>::new(
            self.file.try_clone().unwrap(),
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

        let victim = FileMapIterator::<'a, K, V>::new(
            self.file.try_clone().unwrap(),
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

        let file_size = self.file.metadata().unwrap().len();
        let max_size =
            self.capacity as u64 * (FileMapElement::<K, V>::size()) as u64;

        // Find a victim to evict: Either an element with the same key
        // or the minimum element.
        let mut victim: Option<(u64, (K, V))> = None;
        // If there are holes and the victim does not have the same key
        // Then we insert in a whole.
        let mut spot: Option<u64> = None;

        // We start walking the file in search for the same key, holes and
        // potential victims.
        // Everything is one in one pass.
        for (off, opt) in FileMapIterator::<'a, K, V>::new(
            self.file.try_clone().unwrap(),
            FileMapIteratorPath::PhantomPath,
            self.buffer_size,
        ) {
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
                if file_size < max_size {
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
                } else if file_size >= max_size {
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
    K: 'a + Sized + Eq,
    V: 'a + Sized + Ord,
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
pub struct FileMapValue<K, V>
where
    K: Sized,
    V: Sized,
{
    file: File,
    offset: u64,
    key: K,
    value: V,
}

impl<K: Sized, V: Sized> FileMapValue<K, V> {
    fn new(file_handle: &File, offset: u64, key: K, value: V) -> Self {
        FileMapValue {
            file: file_handle.try_clone().unwrap(),
            offset: offset,
            key: key,
            value: value,
        }
    }
}

impl<K: Sized, V: Sized> Deref for FileMapValue<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K: Sized, V: Sized> DerefMut for FileMapValue<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<K: Sized, V: Sized> Drop for FileMapValue<K, V> {
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
    K: Sized + Eq + 'a,
    V: Sized + Ord + 'a,
{
    type Item = FileMapValue<K, V>;
    fn get(&'a mut self, key: &K) -> Option<Self::Item> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        FileMapIterator::<'a, K, V>::new(
            self.file.try_clone().unwrap(),
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
            FileMapElement::<usize, usize>::write(&mut file, k, v)
                .unwrap();
        }
        file.flush().unwrap();

        // Read elements from file.
        let mut buf = vec![
            0u8;
            input.len()
                * FileMapElement::<usize, usize>::size()
        ];

        (&file).seek(SeekFrom::Start(0)).unwrap();
        let output: Vec<(usize, usize)> = unsafe {
            FileMapElement::<usize, usize>::read(
                &mut file,
                buf.as_mut_slice(),
            )
            .unwrap()
            .into_iter()
            .map(|(_, opt)| opt.unwrap())
            .collect()
        };
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
        remove_file("test_filemap").unwrap();
    }
}

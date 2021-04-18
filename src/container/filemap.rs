use crate::container::{Container, Get};
use crate::marker::Packed;
use tempfile::{NamedTempFile, TempPath};

use std::{
    cmp::Eq,
    fs::{remove_file, File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ops::{Deref, DerefMut, Drop},
    path::{Path, PathBuf},
    slice,
};

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
    pub fn size() -> usize {
        size_of::<FileMapElement<K, V>>()
    }

    /// Create a new valid initialized element.
    fn new(key: K, value: V) -> Self {
        FileMapElement {
            set: true,
            key: key,
            value: value,
        }
    }

    /// Discard this element and output its key
    /// and value.
    pub fn into_kv(self) -> (K, V) {
        (self.key, self.value)
    }

    /// Read a file at current position and output
    /// the element contained at this position.
    /// If the boolean flag `set` from the byte
    /// stream read is not set then the function returns None
    /// to denote that it read an invalid representation.
    /// SAFETY:
    /// 1. Reading a stream of bytes that does not represent an
    /// actual FileMapElement of the same type will result in undefined
    /// behaviour.
    /// 2. If the key type `K` or value type `V` of this element
    /// contains pointers that do not point to a valid initialized
    /// memory area then the behaviour of this function is also undefined.
    pub fn read(f: &mut File) -> Result<Option<Self>> {
        let mut uninit = MaybeUninit::<Self>::uninit();

        // SAFETY: uninit as enough space to fit Self bytes
        let s = unsafe {
            slice::from_raw_parts_mut(
                uninit.as_mut_ptr() as *mut u8,
                size_of::<Self>(),
            )
        };

        match f.read(s) {
            Ok(s) => {
                if s < size_of::<Self>() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "End of File",
                    ))
                } else {
                    // This the unsafe part.
                    // What is read in file must be absolutely
                    // initialized either with `set` flag to false
                    // or with a valid file element.
                    let ret = unsafe { uninit.assume_init() };
                    if ret.set {
                        Ok(Some(ret))
                    } else {
                        Ok(None)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Write initialized FileMapElement to file.
    pub fn write(&self, f: &mut File) -> Result<usize> {
        // SAFETY: slice representation is safe because self is
        // initialized.
        let s = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                size_of::<Self>(),
            )
        };
        f.write(s)
    }

    pub fn write_kv(
        f: &mut File,
        key: &mut K,
        value: &mut V,
    ) -> Result<usize> {
        let uninit = MaybeUninit::<FileMapElement<K, V>>::uninit();
        // SAFETY: Initialization below
        let mut init = unsafe { uninit.assume_init() };
        init.set = true;
        unsafe {
            std::mem::swap(&mut init.key, key);
            std::mem::swap(&mut init.value, value);
        };
        let out = (&init).write(f);

        // Restore input.
        unsafe {
            std::mem::swap(&mut init.key, key);
            std::mem::swap(&mut init.value, value);
        };
        out
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
///     FileMap::new("example_filemap", 2, false).unwrap()
/// };
/// assert!(container.push(0, 0).is_none());
/// assert!(container.push(1, 1).is_none());
/// assert!(container.push(2, 2).unwrap() == (1,1));
/// ```
pub struct FileMap {
    file: File,
    path: PathBuf,
    persistant: bool,
    capacity: usize,
}

impl Drop for FileMap {
    fn drop(&mut self) {
        if self.persistant {
            remove_file(&self.path).unwrap();
        }
    }
}

impl FileMap {
    /// Invalidate one FileMapElement in a FileMap file.
    fn zero(file: &mut File) -> Result<usize> {
        let s: u8 = 0;
        file.write(slice::from_ref(&s))
    }

    /// Read File Represented by this [`FileMap`](struct.FileMap.html)
    /// and look for key `key`. If the key is found, the function return
    /// owned value matching to the first found `key`. If no `key` is found
    /// this function returns `None`.
    pub fn get<K: Sized + Eq, V: Sized>(&self, key: &K) -> Option<V> {
        let mut f = self.file.try_clone().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        loop {
            match FileMapElement::<K, V>::read(&mut f) {
                Err(_) => break None,
                Ok(None) => (),
                Ok(Some(e)) => {
                    let (k, v) = e.into_kv();
                    if &k == key {
                        break Some(v);
                    }
                }
            }
        }
    }

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
    pub unsafe fn new(
        filename: &str,
        capacity: usize,
        persistant: bool,
    ) -> Result<Self> {
        let pb = PathBuf::from(filename);
        match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&pb)
        {
            Ok(f) => Ok(FileMap {
                file: f,
                path: pb,
                capacity: capacity,
                persistant: persistant,
            }),
            Err(e) => Err(e),
        }
    }
}

//------------------------------------------------------------------------//
// Container impl
//------------------------------------------------------------------------//

pub struct FileMapFlushIterator<K: Sized, V: Sized> {
    file: File,
    // When dropped, the temp file is deleted.
    #[allow(dead_code)]
    path: TempPath,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K: Sized, V: Sized> Iterator for FileMapFlushIterator<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match FileMapElement::<K, V>::read(&mut self.file) {
                Err(_) => return None,
                Ok(Some(e)) => return Some(e.into_kv()),
                Ok(None) => {}
            }
        }
    }
}

impl<'a, K, V> Container<'a, K, V> for FileMap
where
    K: 'a + Sized + Eq,
    V: 'a + Sized + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        let mut count = 0usize;
        let mut file = self.file.try_clone().unwrap();
        file.flush().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        loop {
            match FileMapElement::<K, V>::read(&mut file) {
                Err(_) => break,
                Ok(None) => (),
                Ok(Some(_)) => {
                    count += 1;
                }
            }
        }
        count
    }

    fn contains(&self, key: &K) -> bool {
        self.file.try_clone().unwrap().flush().unwrap();
        match self.get::<K, V>(key) {
            None => false,
            Some(_) => true,
        }
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let tmp_path = NamedTempFile::new().unwrap().into_temp_path();
        std::fs::rename(&self.path, AsRef::<Path>::as_ref(&tmp_path))
            .unwrap();

        self.file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&self.path)
            .unwrap();

        let tp = String::from(
            AsRef::<Path>::as_ref(&tmp_path).to_str().unwrap(),
        );
        Box::new(FileMapFlushIterator {
            file: OpenOptions::new().read(true).open(tp).unwrap(),
            path: tmp_path,
            unused_k: PhantomData,
            unused_v: PhantomData,
        })
    }

    fn take(&mut self, key: &K) -> Option<V> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();
        loop {
            match FileMapElement::<K, V>::read(&mut self.file) {
                Err(_) => break None,
                Ok(None) => (),
                Ok(Some(e)) => {
                    let (k, v) = e.into_kv();
                    if &k == key {
                        self.file
                            .seek(SeekFrom::Current(
                                -1 * (FileMapElement::<K, V>::size()
                                    as i64),
                            ))
                            .unwrap();
                        FileMap::zero(&mut self.file).unwrap();
                        break Some(v);
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        self.file.set_len(0).unwrap()
    }

    fn pop(&mut self) -> Option<(K, V)> {
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        let file_size = self.file.metadata().unwrap().len();
        let mut victim: Option<(u64, (K, V))> = None;

        for off in (0..file_size).step_by(FileMapElement::<K, V>::size()) {
            victim = match (
                &victim,
                FileMapElement::<K, V>::read(&mut self.file),
            ) {
                (_, Err(_)) => victim,
                (_, Ok(None)) => victim,
                (None, Ok(Some(e))) => Some((off, e.into_kv())),
                (Some((_, (_, rv))), Ok(Some(e))) => {
                    let (k, r) = e.into_kv();
                    if rv < &r {
                        Some((off, (k, r)))
                    } else {
                        victim
                    }
                }
            }
        }

        match victim {
            None => None,
            Some((off, (k, r))) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMap::zero(&mut self.file).unwrap();
                Some((k, r))
            }
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        // Flush any outstanding write because we want to read the whole
        // file.
        self.file.flush().unwrap();

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
        // Everything is done in one pass.
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();
        for off in (0..file_size).step_by(FileMapElement::<K, V>::size()) {
            match FileMapElement::<K, V>::read(&mut self.file) {
                // We can't look further in the file.
                Err(_) => break,
                // There is a hole, a potential spot for insertion.
                Ok(None) => {
                    spot = Some(off);
                }
                // There is an element. Does it have the same key or is it
                // a better victim?
                Ok(Some(e)) => {
                    let (k, v) = e.into_kv();
                    // We found the same key, we stop here with the victim to
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
            // Then we append element at the end of the file.
            (None, None) => {
                let e = FileMapElement::new(key, reference);
                self.file.seek(SeekFrom::End(0)).unwrap();
                (&e).write(&mut self.file).unwrap();
                None
            }
            // No victim but a spot, then insert in the spot.
            (None, Some(offset)) => {
                let e = FileMapElement::new(key, reference);
                self.file.seek(SeekFrom::Start(offset)).unwrap();
                (&e).write(&mut self.file).unwrap();
                None
            }
            // A victim and a spot! If the victim has the same key then
            // We evict the victim, else we fill the spot
            (Some((off, (k, v))), Some(offset)) => {
                if k == key {
                    let e = FileMapElement::new(key, reference);
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    (&e).write(&mut self.file).unwrap();
                    Some((k, v))
                } else {
                    let e = FileMapElement::new(key, reference);
                    self.file.seek(SeekFrom::Start(offset)).unwrap();
                    (&e).write(&mut self.file).unwrap();
                    None
                }
            }
            // A victim and no spot.
            // If the container is full, then we replace the victim else
            // we append at the end of the file.
            (Some((off, (k, v))), None) => {
                let e = FileMapElement::new(key, reference);
                if file_size >= max_size {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    (&e).write(&mut self.file).unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::End(0)).unwrap();
                    (&e).write(&mut self.file).unwrap();
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
        FileMapElement::<K, V>::write_kv(
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

        let file_size = self.file.metadata().unwrap().len();
        for off in (0..file_size).step_by(FileMapElement::<K, V>::size()) {
            match FileMapElement::<K, V>::read(&mut self.file) {
                Err(_) => return None,
                Ok(None) => (),
                Ok(Some(e)) => {
                    let (k, v) = e.into_kv();
                    if &k == key {
                        return Some(FileMapValue::<K, V>::new(
                            &self.file, off, k, v,
                        ));
                    }
                }
            }
        }
        None
    }
}

//------------------------------------------------------------------------//
// Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::{FileMap, FileMapElement};
    use crate::container::Container;
    use std::fs::{remove_file, File, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};

    fn setup(filename: &str) -> File {
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap()
    }

    fn teardown(filename: &str) {
        remove_file(filename).unwrap();
    }

    fn write_filemap_element(file: &mut File) {
        file.seek(SeekFrom::Start(0)).unwrap();
        for i in 0usize..16usize {
            FileMapElement::new(i.clone(), i.clone())
                .write(file)
                .unwrap();
        }
        file.flush().unwrap();
    }

    fn read_filemap_element(
        file: &mut File,
    ) -> Vec<(usize, Option<FileMapElement<usize, usize>>)> {
        file.seek(SeekFrom::Start(0)).unwrap();

        (0..16)
            .map(|i| {
                (i, FileMapElement::<usize, usize>::read(file).unwrap())
            })
            .collect()
    }

    #[test]
    fn test_filemap_element() {
        let filename: &str = "test_filemap_element";
        let mut file = setup(filename);
        write_filemap_element(&mut file);
        for (i, e) in read_filemap_element(&mut file) {
            let (k, v) = e.unwrap().into_kv();
            assert_eq!(k, i);
            assert_eq!(v, i);
        }
        teardown(filename);
    }

    #[test]
    fn test_filemap() {
        let mut fm =
            unsafe { FileMap::new("test_filemap", 10, false).unwrap() };
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

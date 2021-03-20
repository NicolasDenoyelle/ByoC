use crate::container::{Container, Packed};
use std::{
    cmp::Eq,
    fs::{remove_file, File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ops::Drop,
    path::PathBuf,
    slice,
    string::String,
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
    pub fn write(self, f: &mut File) -> Result<usize> {
        // SAFETY: slice representation is safe because self is
        // initialized.
        let s = unsafe {
            slice::from_raw_parts(
                &self as *const _ as *const u8,
                size_of::<Self>(),
            )
        };
        f.write(s)
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
/// use cache::container::Container;
/// use cache::container::sequential::FileMap;
///
/// let mut container = unsafe {
///     FileMap::new("example_filemap", 2, false).unwrap()
/// };
/// assert!(container.push(0usize, 0usize).is_none());
/// assert!(container.push(1usize, 1usize).is_none());
/// assert!(container.push(2usize, 2usize).unwrap().1 == 1usize);
/// ```
pub struct FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    file: File,
    persistant: Option<String>,
    capacity: usize,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> Drop for FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    fn drop(&mut self) {
        match &self.persistant {
            Some(filename) => {
                remove_file(filename).unwrap();
            }
            None => (),
        }
    }
}

impl<K, V> FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    const ELEMENT_SIZE: usize = size_of::<FileMapElement<K, V>>();

    /// Invalidate one FileMapElement in a FileMap file.
    fn zero(file: &mut File) -> Result<usize> {
        let s: u8 = 0;
        file.write(slice::from_ref(&s))
    }

    /// Read File Represented by this [`FileMap`](struct.FileMap.html)
    /// and look for key `key`. If the key is found, the function return
    /// owned value matching to the first found `key`. If no `key` is found
    /// this function returns `None`.
    pub fn get(&self, key: &K) -> Option<V> {
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
        match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(PathBuf::from(filename))
        {
            Ok(f) => Ok(FileMap {
                file: f,
                capacity: capacity,
                persistant: if !persistant {
                    Some(String::from(filename))
                } else {
                    None
                },
                unused_k: PhantomData,
                unused_v: PhantomData,
            }),
            Err(e) => Err(e),
        }
    }
}

//----------------------------------------------------------------------------//
// Iterator of FileMap elements.
//----------------------------------------------------------------------------//

pub struct FileMapIterator<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    file: File,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> FileMapIterator<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    pub fn new(mut f: File) -> Result<Self> {
        f.seek(SeekFrom::Start(0))?;
        Ok(FileMapIterator {
            file: f,
            unused_k: PhantomData,
            unused_v: PhantomData,
        })
    }
}

impl<K, V> Iterator for FileMapIterator<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match FileMapElement::<K, V>::read(&mut self.file) {
                Err(_) => break None,
                Ok(Some(e)) => break Some(e.into_kv()),
                Ok(None) => (),
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let f = self.file.try_clone().unwrap();
        FileMapIterator::<K, V>::new(f).unwrap()
    }
}

impl<'a, K, V> IntoIterator for &'a mut FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let f = self.file.try_clone().unwrap();
        FileMapIterator::<K, V>::new(f).unwrap()
    }
}

//----------------------------------------------------------------------------//
// Container impl
//----------------------------------------------------------------------------//

impl<K, V> Container<K, V> for FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        self.into_iter().count()
    }

    fn contains(&self, key: &K) -> bool {
        match self.get(key) {
            None => false,
            Some(_) => true,
        }
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
                                -1 * (FileMap::<K, V>::ELEMENT_SIZE as i64),
                            ))
                            .unwrap();
                        FileMap::<K, V>::zero(&mut self.file).unwrap();
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

        for off in (0..file_size).step_by(FileMap::<K, V>::ELEMENT_SIZE) {
            victim =
                match (&victim, FileMapElement::<K, V>::read(&mut self.file)) {
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
                FileMap::<K, V>::zero(&mut self.file).unwrap();
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
            self.capacity as u64 * (FileMap::<K, V>::ELEMENT_SIZE) as u64;

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
        for off in (0..file_size).step_by(FileMap::<K, V>::ELEMENT_SIZE) {
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
                e.write(&mut self.file).unwrap();
                None
            }
            // No victim but a spot, then insert in the spot.
            (None, Some(offset)) => {
                let e = FileMapElement::new(key, reference);
                self.file.seek(SeekFrom::Start(offset)).unwrap();
                e.write(&mut self.file).unwrap();
                None
            }
            // A victim and a spot! If the victim has the same key then
            // We evict the victim, else we fill the spot
            (Some((off, (k, v))), Some(offset)) => {
                if k == key {
                    let e = FileMapElement::new(key, reference);
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    e.write(&mut self.file).unwrap();
                    Some((k, v))
                } else {
                    let e = FileMapElement::new(key, reference);
                    self.file.seek(SeekFrom::Start(offset)).unwrap();
                    e.write(&mut self.file).unwrap();
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
                    e.write(&mut self.file).unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::End(0)).unwrap();
                    e.write(&mut self.file).unwrap();
                    None
                }
            }
        }
    }
}

impl<K, V> Packed<K, V> for FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized + Ord,
{
}

//----------------------------------------------------------------------------//
// Tests
//----------------------------------------------------------------------------//

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
            .map(|i| (i, FileMapElement::<usize, usize>::read(file).unwrap()))
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
        let mut fm = unsafe {
            FileMap::<usize, usize>::new("test_filemap", 10, false).unwrap()
        };
        // Push test
        for i in (0usize..10usize).rev() {
            assert!(fm.push(i, i).is_none());
        }
        // Pop test
        assert_eq!(fm.pop().unwrap().0, 9usize);
        // Contains test
        for i in 0usize..9usize {
            assert!(fm.contains(&i));
        }
        let i = 9usize;
        assert!(!fm.contains(&i));

        // Iteration test
        let mut it = fm.into_iter();
        for i in (0usize..9usize).rev() {
            match it.next() {
                None => panic!("Premature end of iteration"),
                Some((k, _)) => {
                    assert_eq!(k, i);
                }
            }
        }
        assert!(it.next().is_none());

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
                assert_eq!(k, 4usize);
            }
        }

        // Test empty container.
        assert_eq!(fm.pop().unwrap().0, 11usize);
        for i in (0usize..9usize).rev() {
            assert_eq!(fm.pop().unwrap().0, i);
        }
        assert!(fm.pop().is_none());
        assert_eq!(fm.count(), 0);
    }
}

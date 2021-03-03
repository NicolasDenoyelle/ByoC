use crate::{container::Container, reference::Reference};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Error as IOError, ErrorKind, Result as IOResult};
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use std::path::PathBuf;
use std::slice;

#[derive(Debug)]
pub struct OutOfBoundError {
    lower_bound: usize,
    upper_bound: usize,
    value: usize,
}

impl fmt::Display for OutOfBoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Out of bounds [{}:{}] error: {}",
            self.lower_bound, self.upper_bound, self.value
        ))
    }
}

impl Error for OutOfBoundError {}

#[repr(C, packed)]
pub struct FileMapElement<K, V>
where
    K: Sized,
    V: Sized,
{
    set: bool,
    key: K,
    value: V,
}

#[allow(dead_code)]
impl<K, V> FileMapElement<K, V>
where
    K: Sized,
    V: Sized,
{
    pub fn new(key: K, value: V) -> Self {
        FileMapElement {
            set: true,
            key: key,
            value: value,
        }
    }

    pub fn into_kv(self) -> (K, V) {
        (self.key, self.value)
    }

    pub fn read(f: &mut File) -> IOResult<Option<Self>> {
        let mut uninit = MaybeUninit::<Self>::uninit();
        // SAFETY: Fully initialize on reading file.
        // If file reading fails, ret is not used.
        // If file reading succeeded and ret is not initialized in file,
        // then field (set) is set to false because file is zero
        // initialized.
        // SAFETY: uninit as enough space to fit Self bytes
        let s = unsafe {
            slice::from_raw_parts_mut(
                uninit.as_mut_ptr() as *mut u8,
                size_of::<Self>(),
            )
        };

        match f.read(s) {
            Ok(_) => {
                let ret = unsafe { uninit.assume_init() };
                if ret.set {
                    Ok(Some(ret))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn write(&self, f: &mut File) -> IOResult<usize> {
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
}

pub struct FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    file: File,
    capacity: usize,
    hasher: H,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

#[allow(dead_code)]
impl<K, V, H> FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    const ELEMENT_SIZE: usize = size_of::<FileMapElement<K, V>>();

    fn index(&self, key: &K) -> u64 {
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        hasher.finish() % self.capacity as u64
    }

    fn offset_of(&self, key: &K) -> Result<u64, OutOfBoundError> {
        let i = self.index(key);
        let size = self.file.metadata().unwrap().len();
        let offset = i
            .checked_mul(FileMap::<K, V, H>::ELEMENT_SIZE as u64)
            .unwrap();

        if offset >= size {
            Err(OutOfBoundError {
                lower_bound: 0usize,
                upper_bound: size as usize,
                value: offset as usize,
            })
        } else {
            Ok(offset)
        }
    }

    fn zero(&mut self, len: u64) -> IOResult<usize> {
        let zero: [u8; 512] = [0; 512];
        let n = len / 4096;
        for _ in 0..n {
            self.file.write(&zero)?;
        }

        let rem = (len % 4096) / size_of::<u8>() as u64;
        self.file.write(&zero[..rem as usize])
    }

    fn extend(&mut self, len: u64) -> IOResult<u64> {
        let new_size = (self.file.metadata().unwrap().len())
            .checked_add(len)
            .expect("u64 overflow when computing new file size.");
        let max_size = (self.capacity as u64)
            .checked_mul(FileMap::<K, V, H>::ELEMENT_SIZE as u64)
            .unwrap();

        if new_size > max_size {
            IOResult::Err(IOError::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "Cannot extend past FileMap past capcacity {}",
                    &self.capacity
                ),
            ))
        } else {
            self.file.seek(SeekFrom::End(0))?;
            match self.zero(len) {
                Err(e) => Err(e),
                Ok(i) => Ok(i as u64),
            }
        }
    }

    fn seek_const(&self, key: &K) -> Result<IOResult<File>, OutOfBoundError> {
        match self.offset_of(key) {
            Err(e) => Err(e),
            Ok(offset) => match self.file.try_clone() {
                Err(e) => Ok(Err(e)),
                Ok(mut f) => match f.seek(SeekFrom::Start(offset)) {
                    Ok(_) => Ok(IOResult::<File>::Ok(f)),
                    Err(e) => Ok(IOResult::<File>::Err(e)),
                },
            },
        }
    }

    fn seek(&mut self, key: &K) -> IOResult<u64> {
        match self.offset_of(key) {
            Err(bounds) => {
                let size = self.file.metadata().unwrap().len() as usize;
                let bound_size =
                    bounds.value + FileMap::<K, V, H>::ELEMENT_SIZE;
                let max_size = self.capacity * FileMap::<K, V, H>::ELEMENT_SIZE;
                assert!(bound_size <= max_size);

                let new_size = match size.checked_mul(2) {
                    Some(s) => {
                        if s > bound_size {
                            s
                        } else {
                            bound_size
                        }
                    }
                    None => bound_size,
                };
                match self.extend(new_size as u64) {
                    Ok(_) => self
                        .file
                        .seek(SeekFrom::Start(self.offset_of(key).unwrap())),
                    e => e,
                }
            }
            Ok(offset) => self.file.seek(SeekFrom::Start(offset)),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        match self.seek_const(&key) {
            Err(_) => None,
            Ok(Err(_)) => None,
            Ok(Ok(mut f)) => match FileMapElement::<K, V>::read(&mut f) {
                Ok(Some(element)) => Some(element.value),
                Ok(None) => None,
                Err(_) => None,
            },
        }
    }

    pub fn new(
        filename: &str,
        capacity: usize,
        hasher: H,
    ) -> Result<Self, IOError> {
        match File::create(PathBuf::from(filename)) {
            Ok(f) => Ok(FileMap {
                file: f,
                capacity: capacity,
                hasher: hasher,
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
    K: Sized,
    V: Sized,
{
    file: File,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> FileMapIterator<K, V>
where
    K: Sized + Hash,
    V: Sized,
{
    pub fn new<H: Hasher + Clone>(fmap: &FileMap<K, V, H>) -> IOResult<Self> {
        let mut f = fmap.file.try_clone()?;
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
    K: Sized + Hash,
    V: Sized,
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

impl<K, V, H> IntoIterator for FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        FileMapIterator::<K, V>::new(&self).unwrap()
    }
}

impl<'a, K, V, H> IntoIterator for &'a FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        FileMapIterator::<K, V>::new(self).unwrap()
    }
}

impl<'a, K, V, H> IntoIterator for &'a mut FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        FileMapIterator::<K, V>::new(self).unwrap()
    }
}

//----------------------------------------------------------------------------//
// Container impl
//----------------------------------------------------------------------------//

impl<K, V, R, H> Container<K, V, R> for FileMap<K, R, H>
where
    K: Ord + Sized + Hash,
    V: Sized,
    R: Reference<V>,
    H: Hasher + Clone,
{
    fn capacity(&self) -> usize {
        let size = self.file.metadata().unwrap().len();
        (size / FileMap::<K, R, H>::ELEMENT_SIZE as u64) as usize
    }

    fn count(&self) -> usize {
        self.into_iter().count()
    }

    fn contains(&self, key: &K) -> bool {
        match self.seek_const(key) {
            Err(_) => false,
            Ok(Err(_)) => false,
            Ok(Ok(mut f)) => match FileMapElement::<K, V>::read(&mut f) {
                Err(_) => false,
                Ok(Some(_)) => true,
                Ok(None) => false,
            },
        }
    }

    fn take(&mut self, key: &K) -> Option<R> {
        match self.seek_const(key) {
            Err(_) => None,
            Ok(Err(_)) => None,
            Ok(Ok(mut f)) => match FileMapElement::<K, R>::read(&mut f) {
                Err(_) => None,
                Ok(Some(e)) => {
                    self.seek(key).unwrap();
                    self.zero(size_of::<FileMapElement<K, R>>() as u64)
                        .unwrap();
                    Some(e.value)
                }
                Ok(None) => None,
            },
        }
    }

    fn clear(&mut self) {
        self.file.set_len(0).unwrap()
    }

    fn pop(&mut self) -> Option<(K, R)> {
        match self.into_iter().max_by(|a, b| (&a.0).cmp(&b.0)) {
            None => None,
            Some((k, v)) => {
                self.take(&k).unwrap();
                Some((k, v))
            }
        }
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        self.seek(&key).unwrap();
        let ret = match FileMapElement::<K, R>::read(&mut self.file) {
            Err(_) => None,
            Ok(Some(e)) => Some(e.into_kv()),
            Ok(None) => None,
        };
        self.seek(&key).unwrap();
        FileMapElement::new(key, reference)
            .write(&mut self.file)
            .unwrap();
        ret
    }
}

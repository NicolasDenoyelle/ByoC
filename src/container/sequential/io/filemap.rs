use crate::{container::Container, reference::Reference};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
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

    pub fn read(f: &mut File) -> std::io::Result<Option<Self>> {
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

    pub fn write(&self, f: &mut File) -> std::io::Result<usize> {
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
    file_handle: File,
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
    fn offset_of(&self, key: &K) -> Result<usize, OutOfBoundError> {
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let mut offset = hasher.finish() as usize;
        offset = offset % self.capacity;
        offset = offset * size_of::<FileMapElement<K, V>>();
        let size = self.capacity * size_of::<FileMapElement<K, V>>();

        if offset >= size {
            Err(OutOfBoundError {
                lower_bound: 0usize,
                upper_bound: size,
                value: offset,
            })
        } else {
            Ok(size)
        }
    }

    fn zero(file_handle: &mut File, size: usize) {
        let zero: [u8; 512] = [0; 512];
        let n = size / 4096usize;
        for _ in 0..n {
            file_handle
                .write(&zero)
                .expect("Error initializing file with zeroes");
        }

        let rem = (size % 4096) / size_of::<u8>();
        file_handle
            .write(&zero[..rem])
            .expect("Error initializing file with zeroes");
    }

    pub fn new(
        directory: &str,
        id: usize,
        capacity: usize,
        hasher: H,
    ) -> Result<Self, std::io::Error> {
        let mut path = PathBuf::from(directory);
        path.push(format!("{}", id));

        match File::create(path) {
            Ok(mut f) => {
                Self::zero(&mut f, capacity);
                Ok(FileMap {
                    file_handle: f,
                    capacity: capacity,
                    hasher: hasher,
                    unused_k: PhantomData,
                    unused_v: PhantomData,
                })
            }
            Err(e) => Err(e),
        }
    }
}

//----------------------------------------------------------------------------//
// TODO: Impl iter trait for going through elements.
//----------------------------------------------------------------------------//
// impl<K, V, R, H> Container<K, V, R> for FileMap<K, R, H>
// where
//     K: Ord + Sized,
//     V: Sized,
//     R: Reference<V>,
//     H: Hash,
// {
//     fn capacity(&self) -> usize {
//         self.capacity
//     }

//     fn count(&self) -> usize {
//         let mut count = 0usize;
//         let mut f = self
//             .file_handle
//             .try_clone()
//             .expect("Cannot clone file handle.");
//         f.seek(SeekFrom::Start(0));
//         loop {
//             match FileMapElement::<K, V>::read(&mut f) {
//                 Err(_) => break,
//                 Ok(Some(e)) => count += 1,
//                 Ok(None) => (),
//             }
//         }
//         count
//     }

// fn contains(&self, key: &K) -> bool {
//     let mut f = self
//         .file_handle
//         .try_clone()
//         .expect("Cannot clone file handle.");
//     match self.offset_of(key) {
//         Err(_) => return false,
//         Ok(offset) => match f.seek(SeekFrom::Start(offset as u64)) {
//             Err(_) => false,
//             Ok(_) => match FileMapElement::<K, V>::read(&mut f) {
//                 Err(_) => false,
//                 Ok(Some(_)) => true,
//                 Ok(None) => false,
//             },
//         },
//     }
// }

// fn take(&mut self, key: &K) -> Option<R> {
//     let mut f = self
//         .file_handle
//         .try_clone()
//         .expect("Cannot clone file handle.");
//     match self.offset_of(key) {
//         Err(_) => return None,
//         Ok(offset) => match f.seek(SeekFrom::Start(offset as u64)) {
//             Err(_) => None,
//             Ok(_) => match FileMapElement::<K, R>::read(&mut f) {
//                 Err(_) => None,
//                 Ok(Some(e)) => Some(e.value),
//                 Ok(None) => None,
//             },
//         },
//     }
// }

// fn clear(&mut self) {
//     FileMap::<K, R, H>::zero(
//         &mut self.file_handle,
//         self.capacity * size_of::<FileMapElement<K, R>>(),
//     )
// }

//     fn pop(&mut self) -> Option<(K, R)>;
//     fn push(&mut self, key: K, reference: R) -> Option<(K, R)>;
// }

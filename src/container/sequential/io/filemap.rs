use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::size_of;
use std::path::PathBuf;

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

pub struct FileMapElement<K, V>
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
    pub fn new(key: K, value: V) -> Self {
        FileMapElement {
            set: true,
            key: key,
            value: value,
        }
    }
}

impl<K, V> Default for FileMapElement<K, V>
where
    K: Sized + Default,
    V: Sized + Default,
{
    fn default() -> Self {
        FileMapElement {
            set: false,
            key: K::default(),
            value: V::default(),
        }
    }
}

pub struct FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    start_key: usize,
    file_handle: File,
    size: usize,
    hasher: H,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V, H> FileMap<K, V, H>
where
    K: Sized + Hash,
    V: Sized,
    H: Hasher + Clone,
{
    fn start_offset(&self) -> usize {
        self.start_key * size_of::<FileMapElement<K, V>>()
    }
    fn end_offset(&self) -> usize {
        (self.start_key + self.size) * size_of::<FileMapElement<K, V>>()
    }
    fn offset_of(self, key: &K) -> Result<usize, OutOfBoundError> {
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let value =
            hasher.finish() as usize * size_of::<FileMapElement<K, V>>();
        let lower_bound = self.start_offset();
        let upper_bound = self.end_offset();

        if value < lower_bound || value >= upper_bound {
            Err(OutOfBoundError {
                lower_bound: lower_bound,
                upper_bound: upper_bound,
                value: value,
            })
        } else {
            Ok(value)
        }
    }

    pub fn new(
        start_key: usize,
        size: usize,
        directory: &str,
        hasher: H,
    ) -> Result<Self, std::io::Error> {
        let mut path = PathBuf::from(directory);
        path.push(format!("{}", start_key));

        match File::create(path) {
            Ok(f) => Ok(FileMap {
                start_key: start_key,
                file_handle: f,
                size: size,
                hasher: hasher,
                unused_k: PhantomData,
                unused_v: PhantomData,
            }),
            Err(e) => Err(e),
        }
    }
}

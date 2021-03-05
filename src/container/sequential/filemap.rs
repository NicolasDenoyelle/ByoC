use crate::{container::Container, reference::Reference};
use std::{
    cmp::{min, Ord},
    fs::File,
    io::{Error as IOError, ErrorKind, Read, Result, Seek, SeekFrom, Write},
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    path::PathBuf,
    slice,
};

#[repr(C, packed)]
struct FileMapElement<K, V>
where
    K: Sized + Ord,
    V: Sized,
{
    set: bool,
    key: K,
    value: V,
}

impl<K, V> FileMapElement<K, V>
where
    K: Sized + Ord,
    V: Sized,
{
    fn new(key: K, value: V) -> Self {
        FileMapElement {
            set: true,
            key: key,
            value: value,
        }
    }

    pub fn into_kv(self) -> (K, V) {
        (self.key, self.value)
    }

    // pub fn from_bytes(b: NonNull<u8>) -> Option<Self> {
    //     let e: Self = unsafe { read(transmute::<_, *const Self>(b.as_ptr())) };

    //     match e.set() {
    //         false => None,
    //         true => Some(e),
    //     }
    // }

    pub fn read(f: &mut File) -> Result<Option<Self>> {
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
}

pub struct FileMap<K, V>
where
    K: Sized + Ord,
    V: Sized,
{
    file: File,
    capacity: usize,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> FileMap<K, V>
where
    K: Sized + Ord,
    V: Sized,
{
    const ELEMENT_SIZE: usize = size_of::<FileMapElement<K, V>>();

    fn zero(file: &mut File, len: u64) -> Result<usize> {
        let zero: [u8; 512] = [0; 512];
        let n = len / 4096;
        for _ in 0..n {
            file.write(&zero)?;
        }

        let rem = (len % 4096) / size_of::<u8>() as u64;
        file.write(&zero[..rem as usize])
    }

    fn extend(&mut self, len: u64) -> Result<u64> {
        let new_size = (self.file.metadata().unwrap().len())
            .checked_add(len)
            .expect("u64 overflow when computing new file size.");
        let max_size = (self.capacity as u64)
            .checked_mul(FileMap::<K, V>::ELEMENT_SIZE as u64)
            .unwrap();

        if new_size > max_size {
            Result::Err(IOError::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "Cannot extend past FileMap past capcacity {}",
                    &self.capacity
                ),
            ))
        } else {
            self.file.seek(SeekFrom::End(0))?;
            match FileMap::<K, V>::zero(&mut self.file, len) {
                Err(e) => Err(e),
                Ok(i) => Ok(i as u64),
            }
        }
    }

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

    pub fn new(filename: &str, capacity: usize) -> Result<Self> {
        match File::create(PathBuf::from(filename)) {
            Ok(f) => Ok(FileMap {
                file: f,
                capacity: capacity,
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
    K: Sized + Ord,
    V: Sized,
{
    file: File,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> FileMapIterator<K, V>
where
    K: Sized + Ord,
    V: Sized,
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
    K: Sized + Ord,
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

impl<K, V> IntoIterator for FileMap<K, V>
where
    K: Sized + Ord,
    V: Sized,
{
    type Item = (K, V);
    type IntoIter = FileMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        FileMapIterator::<K, V>::new(self.file).unwrap()
    }
}

impl<'a, K, V> IntoIterator for &'a FileMap<K, V>
where
    K: Sized + Ord,
    V: Sized,
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
    K: Sized + Ord,
    V: Sized,
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

impl<K, V, R> Container<K, V, R> for FileMap<K, R>
where
    K: Ord + Sized,
    V: Sized,
    R: Reference<V>,
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

    fn take(&mut self, key: &K) -> Option<R> {
        self.file.seek(SeekFrom::Start(0)).unwrap();
        loop {
            match FileMapElement::<K, R>::read(&mut self.file) {
                Err(_) => break None,
                Ok(None) => (),
                Ok(Some(e)) => {
                    let (k, v) = e.into_kv();
                    if &k == key {
                        self.file
                            .seek(SeekFrom::Current(
                                -1 * (FileMap::<K, R>::ELEMENT_SIZE as i64),
                            ))
                            .unwrap();
                        FileMap::<K, R>::zero(
                            &mut self.file,
                            FileMap::<K, R>::ELEMENT_SIZE as u64,
                        )
                        .unwrap();
                        break Some(v);
                    }
                }
            }
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
        let file_size = self.file.metadata().unwrap().len();
        let max_size = self.capacity * (FileMap::<K, R>::ELEMENT_SIZE);

        // If this is the first push, we have to grow file and
        // insert at the begining.
        if file_size == 0 {
            let desired_size = FileMap::<K, R>::ELEMENT_SIZE * 64;
            let size = min(max_size, desired_size) as u64;
            match self.extend(size) {
                Err(_) => panic!("Cannot grow file size."),
                Ok(_) => {
                    self.file.seek(SeekFrom::Start(0)).unwrap();
                    FileMapElement::new(key, reference)
                        .write(&mut self.file)
                        .unwrap();
                    return None;
                }
            }
        }

        // Find a victim to evict: Either an element with the same key
        // or the minimum element.
        let mut victim: Option<(u64, (K, R))> = None;
        // If there are holes and the victim does not have the same key
        // Then we insert in a whole.
        let mut spot: Option<u64> = None;

        // We start walking the file in search for the same key, holes and
        // potential victims.
        for off in (0..file_size).step_by(FileMap::<K, R>::ELEMENT_SIZE) {
            match FileMapElement::<K, R>::read(&mut self.file) {
                // We can't look further in the file.
                Err(_) => break,
                // There is a hole, a potential spot for insertion.
                Ok(None) => {
                    spot = if spot.is_none() { Some(off) } else { spot };
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
                            // There is a hole, we don't care about victims then.
                            (Some(_), vict) => vict,
                            // There is no current victim and no hole then,
                            // This is the current victim.
                            (None, None) => Some((off, (k, v))),
                            // Next victim is the element with max reference.
                            (None, Some((off1, (k1, v1)))) => {
                                if v1 > v {
                                    Some((off1, (k1, v1)))
                                } else {
                                    Some((off, (k, v)))
                                }
                            }
                        }
                    }
                }
            }
        }

        match (victim, spot) {
            // No victim and no spot... Then we have to extend the file
            // To make space.
            (None, None) => {
                match self.extend(min(file_size * 2, max_size as u64)) {
                    Err(_) => panic!("Cannot grow file size."),
                    Ok(_) => {
                        self.file.seek(SeekFrom::Start(file_size)).unwrap();
                        FileMapElement::new(key, reference)
                            .write(&mut self.file)
                            .unwrap();
                        None
                    }
                }
            }
            // No victim but a spot, then insert in the spot.
            (None, Some(offset)) => {
                self.file.seek(SeekFrom::Start(offset)).unwrap();
                FileMapElement::new(key, reference)
                    .write(&mut self.file)
                    .unwrap();
                None
            }
            // A victim and a spot! If the victim has the same key then
            // We evict the victim, else we fill the spot
            (Some((off, (k, v))), Some(offset)) => {
                if k == key {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    FileMapElement::new(key, reference)
                        .write(&mut self.file)
                        .unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::Start(offset)).unwrap();
                    FileMapElement::new(key, reference)
                        .write(&mut self.file)
                        .unwrap();
                    None
                }
            }
            // A victim and no spot. We replace the victim.
            (Some((off, (k, v))), None) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMapElement::new(key, reference)
                    .write(&mut self.file)
                    .unwrap();
                Some((k, v))
            }
        }
    }
}

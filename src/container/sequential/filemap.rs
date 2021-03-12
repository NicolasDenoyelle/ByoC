use crate::{
    container::{Container, Packed},
    reference::Reference,
};
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
            Ok(s) => {
                if s < size_of::<Self>() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "End of File",
                    ))
                } else {
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

impl<K, V, R> Packed<K, V, R> for FileMap<K, R>
where
    K: Sized + Eq,
    V: Sized,
    R: Reference<V> + Sized,
{
}

pub struct FileMap<K, V>
where
    K: Sized + Eq,
    V: Sized,
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
    V: Sized,
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
    V: Sized,
{
    const ELEMENT_SIZE: usize = size_of::<FileMapElement<K, V>>();

    fn zero(file: &mut File) -> Result<usize> {
        let s: u8 = 0;
        file.write(slice::from_ref(&s))
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

    pub fn new(
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
                persistant: if persistant {
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
    V: Sized,
{
    file: File,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V> FileMapIterator<K, V>
where
    K: Sized + Eq,
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
    K: Sized + Eq,
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

impl<'a, K, V> IntoIterator for &'a FileMap<K, V>
where
    K: Sized + Eq,
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
    K: Sized + Eq,
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
    K: Sized + Eq,
    V: Sized,
    R: Reference<V> + Sized,
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
        self.file.flush().unwrap();
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
                        FileMap::<K, R>::zero(&mut self.file).unwrap();
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
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();

        let file_size = self.file.metadata().unwrap().len();
        let mut victim: Option<(u64, (K, R))> = None;

        for off in (0..file_size).step_by(FileMap::<K, R>::ELEMENT_SIZE) {
            match FileMapElement::<K, R>::read(&mut self.file) {
                Err(_) => break,
                Ok(None) => {}
                Ok(Some(e)) => {
                    let (k, r) = e.into_kv();
                    match &victim {
                        None => victim = Some((off, (k, r))),
                        Some((_, (_, rv))) => {
                            if rv < &r {
                                victim = Some((off, (k, r)));
                            }
                        }
                    }
                }
            }
        }

        match victim {
            None => None,
            Some((off, (k, r))) => {
                self.file.seek(SeekFrom::Start(off)).unwrap();
                FileMap::<K, R>::zero(&mut self.file).unwrap();
                Some((k, r))
            }
        }
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        // Flush any outstanding write because we want to read the whole
        // file.
        self.file.flush().unwrap();

        let file_size = self.file.metadata().unwrap().len();
        let max_size =
            self.capacity as u64 * (FileMap::<K, R>::ELEMENT_SIZE) as u64;

        // If this is the first push, we have to grow file and
        // insert at the begining.
        if file_size == 0 {
            FileMapElement::new(key, reference)
                .write(&mut self.file)
                .unwrap();
            return None;
        }

        // Find a victim to evict: Either an element with the same key
        // or the minimum element.
        let mut victim: Option<(u64, (K, R))> = None;
        // If there are holes and the victim does not have the same key
        // Then we insert in a whole.
        let mut spot: Option<u64> = None;

        // We start walking the file in search for the same key, holes and
        // potential victims.
        // Everything is done in one pass.
        self.file.flush().unwrap();
        self.file.seek(SeekFrom::Start(0)).unwrap();
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
                self.file.seek(SeekFrom::End(0)).unwrap();
                FileMapElement::new(key, reference)
                    .write(&mut self.file)
                    .unwrap();
                None
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
            // A victim and no spot.
            // if the container is full, then we replace the victim.
            (Some((off, (k, v))), None) => {
                if file_size >= max_size {
                    self.file.seek(SeekFrom::Start(off)).unwrap();
                    FileMapElement::new(key, reference)
                        .write(&mut self.file)
                        .unwrap();
                    Some((k, v))
                } else {
                    self.file.seek(SeekFrom::End(0)).unwrap();
                    FileMapElement::new(key, reference)
                        .write(&mut self.file)
                        .unwrap();
                    None
                }
            }
        }
    }
}

//----------------------------------------------------------------------------//
// Tests
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::{FileMap, FileMapElement};
    use crate::container::Container;
    use crate::reference::Default;
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
        let mut fm =
            FileMap::<usize, Default<usize>>::new("test_filemap", 10, false)
                .unwrap();
        // Push test
        for i in (0usize..10usize).rev() {
            assert!(fm.push(i, Default::new(i)).is_none());
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
        assert!(fm.push(9usize, Default::new(9usize)).is_none());
        match fm.push(11usize, Default::new(11usize)) {
            None => panic!("Full filemap not popping."),
            Some((k, _)) => {
                assert_eq!(k, 9usize);
            }
        }

        // Test pop on push of an existing key.
        match fm.push(4usize, Default::new(4usize)) {
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

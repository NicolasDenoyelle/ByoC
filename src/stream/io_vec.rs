//! Array implementation above a
//! [stream](../utils/stream/trait.Stream.html) and
//! utils for reading and writing a stream in fixed sized chunks.

use crate::stream::Stream;
use serde::{de::DeserializeOwned, Serialize};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::vec::Vec;

//----------------------------------------------------------------------------//
// In-memory representation of a chunk.
//----------------------------------------------------------------------------//

#[derive(Debug)]
pub enum IOError {
    /// Error returned by call to `seek()` from `std::io::Seek` trait.
    Seek(std::io::Error),
    /// Error returned by call to `read()` from `std::io::Read` trait.
    Read(std::io::Error),
    /// Error returned by call to `write()` from `std::io::Write` trait.
    Write(std::io::Error),
    /// Error returned by lz4 encoder builder.
    Encode(std::io::Error),
    /// Error returned by lz4 decoder.
    Decode(std::io::Error),
    /// Error returned by call to `serialize()` from `bincode::serialize()`.
    Serialize(bincode::Error),
    /// Error returned by call to `deserialize()` from `bincode::deserialize()`.
    Deserialize(bincode::Error),
    /// Error related to some size.
    InvalidSize,
}

/// Result type of [`byoc::utils::io`](index.html)
/// See [`IOError`](enum.IOError.html).
pub type IOResult<T> = Result<T, IOError>;

/// A dynamically sized chunk of data.
/// The size is set once at initialization and can never be changed.
pub struct IOChunk {
    buf: Vec<u8>,
}

impl IOChunk {
    /// `IOChunk` constructor.
    /// Creates a new chunk filled with 0s.
    /// `size` is the size of the chunk in bytes.
    pub fn new(size: usize) -> Self {
        IOChunk {
            buf: vec![0u8; size],
        }
    }

    /// Read `size` bytes from a `stream` and copy it into a new chunk.
    pub fn from_stream<F: Read + Seek>(
        size: usize,
        stream: &mut F,
    ) -> IOResult<Option<Self>> {
        let mut buf = vec![0u8; size];
        match stream.read(buf.as_mut_slice()) {
            Ok(len) => {
                if len < size {
                    Ok(None)
                } else {
                    Ok(Some(IOChunk { buf }))
                }
            }
            Err(e) => Err(IOError::Read(e)),
        }
    }

    /// Serialize an `item` into bytes stored inside this chunk.
    /// If the object serialized size exceeds the chunnk size,
    /// the error [`InvalidSizeError`](enum.IOError.html) is returned.
    /// If the object serialization fails,
    /// the error [`SerializeError`](enum.IOError.html) is returned.
    pub fn serialize<T: Serialize>(&mut self, item: &T) -> IOResult<()> {
        match bincode::serialize(item) {
            Err(e) => Err(IOError::Serialize(e)),
            Ok(mut v) => {
                if v.len() > self.buf.capacity() {
                    Err(IOError::InvalidSize)
                } else {
                    v.resize(self.buf.len(), 0u8);
                    self.buf = v;
                    Ok(())
                }
            }
        }
    }

    /// Write the content of a chunk into a stream.
    /// The whole chunk is written to the stream.
    pub fn write<F: Seek + Write>(&self, stream: &mut F) -> IOResult<()> {
        // Write chunk to stream.
        match stream.write(self.buf.as_slice()) {
            Ok(_) => Ok(()),
            Err(e) => Err(IOError::Write(e)),
        }
    }

    /// Deserialize the content of a chunk into an object.
    /// If the deserialization fails,
    /// the error [`DeserializeError`](enum.IOError.html) is returned.
    pub fn deserialize<T: DeserializeOwned>(&mut self) -> IOResult<T> {
        match bincode::deserialize_from(self.buf.as_slice()) {
            Err(e) => Err(IOError::Deserialize(e)),
            Ok(t) => Ok(t),
        }
    }
}

/// Array of fixed size chunks stored into a stream of bytes (e.g a file).
/// The vector primitives will
/// [serialize](../../../bincode/fn.serialize.html)/[deserialize](../../../bincode/fn.deserialize_from.html) items into/from fixed size chunks.
/// If items cannot fit chunks, methods will fail.
pub struct IOVec<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Write + Seek + Clone,
{
    stream: T,
    chunk_size: usize,
    _o: PhantomData<O>,
}

//----------------------------------------------------------------------------//
//  On-disk chunk representation
//----------------------------------------------------------------------------//

/// RAII structure wrapping an item read from a stream
/// that will be written back to the stream at the destruction time.
///
/// The item is written at its initial position in the stream and writing
/// will occur only if `deref_mut()` method from `DerefMut` trait is
/// invoked.
pub struct IOStructMut<T, S>
where
    T: Serialize,
    S: Write + Seek,
{
    item: T,
    stream: S,
    pos: u64,
    is_written: bool,
}

impl<T, S> IOStructMut<T, S>
where
    T: Serialize,
    S: Write + Seek,
{
    /// `IOStructMut` constructor.
    ///
    /// * `stream`: The stream where to write `item` back if
    /// item has written at destruction time.
    /// * `item`: The item to wrap.
    pub fn new(mut stream: S, item: T) -> IOResult<Self> {
        let pos = match stream.seek(SeekFrom::Current(0)) {
            Err(e) => return Err(IOError::Seek(e)),
            Ok(pos) => pos,
        };

        Ok(IOStructMut {
            stream,
            item,
            pos,
            is_written: false,
        })
    }
}

impl<T, S> Drop for IOStructMut<T, S>
where
    T: Serialize,
    S: Write + Seek,
{
    fn drop(&mut self) {
        if self.is_written {
            // Move to chunk position.
            self.stream.seek(SeekFrom::Start(self.pos)).expect(
                "Seek error while writing back IOStructMut to stream.",
            );

            // Serialize object in the stream
            if let Err(e) = bincode::serialize_into::<&mut S, T>(
                &mut self.stream,
                &self.item,
            ) {
                panic!("Serialization error while writing back IOStructMut to stream. {:?}", e);
            }
        }
    }
}

impl<T, S> std::ops::Deref for IOStructMut<T, S>
where
    T: Serialize,
    S: Write + Seek,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T, S> std::ops::DerefMut for IOStructMut<T, S>
where
    T: Serialize,
    S: Write + Seek,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_written = true;
        &mut self.item
    }
}

/// RAII structure wrapping an item read from a stream.
/// The item will not be written back to the stream.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct IOStruct<T> {
    item: T,
}

impl<T> IOStruct<T> {
    /// `IOStruct` constructor.
    pub fn new(item: T) -> Self {
        IOStruct { item }
    }

    pub fn unwrap(self) -> T {
        self.item
    }
}

impl<T> std::ops::Deref for IOStruct<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

//----------------------------------------------------------------------------//
// Iterators implementation
//----------------------------------------------------------------------------//

pub struct IOIter<O, T>
where
    O: DeserializeOwned,
    T: Read + Seek,
{
    stream: BufReader<T>,
    chunk_size: usize,
    _o: PhantomData<O>,
}

impl<O, T> IOIter<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Seek,
{
    pub fn new(stream: BufReader<T>, chunk_size: usize) -> Self {
        IOIter {
            stream,
            chunk_size,
            _o: PhantomData,
        }
    }
}

impl<O, T> Iterator for IOIter<O, T>
where
    O: DeserializeOwned,
    T: Read + Seek,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        match IOChunk::from_stream(self.chunk_size, &mut self.stream) {
            Err(_) => panic!("Read error while iterating stream."),
            Ok(None) => None,
            Ok(Some(mut c)) => match c.deserialize::<O>() {
                Err(_) => {
                    panic!("Deserialize error while iterating stream.")
                }
                Ok(o) => Some(o),
            },
        }
    }
}

pub struct IOVecIter<O, T>
where
    O: DeserializeOwned,
    T: Read + Seek,
{
    stream: BufReader<T>,
    chunk_size: usize,
    _o: PhantomData<O>,
}

impl<O, T> IOVecIter<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Seek,
{
    pub fn new(stream: BufReader<T>, chunk_size: usize) -> Self {
        IOVecIter {
            stream,
            chunk_size,
            _o: PhantomData,
        }
    }
}

impl<O, T> Iterator for IOVecIter<O, T>
where
    O: DeserializeOwned,
    T: Read + Seek,
{
    type Item = IOStruct<O>;

    fn next(&mut self) -> Option<Self::Item> {
        match IOChunk::from_stream(self.chunk_size, &mut self.stream) {
            Err(_) => panic!("Read error while iterating stream."),
            Ok(None) => None,
            Ok(Some(mut c)) => match c.deserialize::<O>() {
                Err(_) => {
                    panic!("Deserialize error while iterating stream.")
                }
                Ok(o) => Some(IOStruct::new(o)),
            },
        }
    }
}

pub struct IOVecIterMut<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Seek + Write,
{
    stream: BufReader<T>,
    chunk_size: usize,
    pos: T,
    _o: PhantomData<O>,
}

impl<O, T> IOVecIterMut<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Seek + Write,
{
    pub fn new(stream: BufReader<T>, chunk_size: usize, pos: T) -> Self {
        IOVecIterMut {
            stream,
            chunk_size,
            pos,
            _o: PhantomData,
        }
    }
}

impl<O, T> Iterator for IOVecIterMut<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Read + Seek + Write + Clone,
{
    type Item = IOStructMut<O, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Create stream at position where to write back the next chunk.
        let current = self.pos.clone();

        // Read stream
        let out = match IOChunk::from_stream(
            self.chunk_size,
            &mut self.stream,
        ) {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(mut c)) => match c.deserialize::<O>() {
                Err(e) => Err(e),
                Ok(o) => match IOStructMut::<O, T>::new(current, o) {
                    Err(e) => Err(e),
                    Ok(s) => {
                        self.pos
                            .seek(SeekFrom::Current(
                                self.chunk_size as i64,
                            ))
                            .expect("Seek error after reading chunk.");
                        Ok(Some(s))
                    }
                },
            },
        };

        match out {
            Ok(o) => o,
            Err(e) => match e {
                IOError::Seek(_) => {
                    panic!("Seek error while iterating stream.")
                }
                IOError::Read(_) => {
                    panic!("Read error while iterating stream.")
                }
                IOError::Deserialize(_) => {
                    panic!("Deserialize error while iterating stream.")
                }
                _ => panic!("Error while iterating stream."),
            },
        }
    }
}

//----------------------------------------------------------------------------//
// IOVec implementation.
//----------------------------------------------------------------------------//

#[allow(dead_code)]
impl<O, T> IOVec<O, T>
where
    O: DeserializeOwned + Serialize,
    T: Stream,
{
    /// `IOVec` constructor.
    ///
    /// ## Arguments:
    ///
    /// * store: The byte stream where items are stored.
    /// * chunk_size: The size of chunks stored in the stream.
    pub fn new(store: T, chunk_size: usize) -> Self {
        IOVec {
            stream: store,
            chunk_size,
            _o: PhantomData,
        }
    }

    /// The number of element in the vector.
    pub fn len(&self) -> IOResult<usize> {
        let mut stream = self.stream.clone();

        match stream.seek(SeekFrom::End(0)) {
            Err(e) => Err(IOError::Seek(e)),
            Ok(pos) => Ok(pos as usize / self.chunk_size),
        }
    }

    /// The total size in bytes.
    pub fn size(&self) -> IOResult<usize> {
        match self.stream.clone().seek(SeekFrom::End(0)) {
            Ok(s) => Ok(s as usize),
            Err(e) => Err(IOError::Seek(e)),
        }
    }

    pub fn is_empty(&self) -> bool {
        let mut stream = self.stream.clone();

        match stream.seek(SeekFrom::Current(0)) {
            Err(_) => true,
            Ok(pos) => pos == 0,
        }
    }

    /// Get read-only access to the `n`th element from the vector.
    /// If the `seek()` on the underlying stream fails,
    /// the error [`SeekError`](enum.IOError.html) is returned.
    /// If the `read()` on the underlying stream fails,
    /// the error [`ReadError`](enum.IOError.html) is returned.
    /// If the vector did not have at least `n` elements, `None` is
    /// returned.
    /// If the element at position `n` cannot be deserialized into target
    /// type, the error [`ReadError`](enum.DeserializeError.html) is
    /// returned.
    /// On success, this method returns an
    /// [`IOStruct`](struct.IOStruct.html) that can be dereferenced
    /// to access the underlying item.
    pub fn get(&self, n: usize) -> IOResult<Option<IOStruct<O>>> {
        let mut stream = self.stream.clone();

        if let Err(e) =
            stream.seek(SeekFrom::Start((n * self.chunk_size) as u64))
        {
            return Err(IOError::Seek(e));
        }

        match IOChunk::from_stream(self.chunk_size, &mut stream) {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(mut c)) => match c.deserialize::<O>() {
                Err(e) => Err(e),
                Ok(item) => Ok(Some(IOStruct::new(item))),
            },
        }
    }

    /// Get read-write access to the `n`th element from the vector.
    /// If the `seek()` on the underlying stream fails,
    /// the error [`SeekError`](enum.IOError.html) is returned.
    /// If the `read()` on the underlying stream fails,
    /// the error [`ReadError`](enum.IOError.html) is returned.
    /// If the vector did not have at least `n` elements, `None` is
    /// returned.
    /// If the element at position `n` cannot be deserialized into target
    /// type, the error [`ReadError`](enum.DeserializeError.html) is
    /// returned.
    /// On success, this method returns
    /// an [`IOStructMut`](struct.IOStructMut.html) that can be
    /// dereferenced to access the underlying item. When this struct
    /// is destroyed, the item is written back to the stream if it has
    /// been modified.
    pub fn get_mut(
        &mut self,
        n: usize,
    ) -> IOResult<Option<IOStructMut<O, T>>> {
        let mut stream = self.stream.clone();

        if let Err(e) =
            stream.seek(SeekFrom::Start((n * self.chunk_size) as u64))
        {
            return Err(IOError::Seek(e));
        }

        match IOChunk::from_stream(self.chunk_size, &mut stream) {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(mut c)) => match c.deserialize::<O>() {
                Err(e) => Err(e),
                Ok(item) => match IOStructMut::new(stream, item) {
                    Err(e) => Err(e),
                    Ok(c) => Ok(Some(c)),
                },
            },
        }
    }

    /// Append an element at the end of the vector.
    pub fn append(&mut self, values: &mut Vec<O>) -> IOResult<()> {
        // Go at the end of the stream
        if let Err(e) = self.stream.seek(SeekFrom::End(0)) {
            return Err(IOError::Seek(e));
        };

        let mut stream = BufWriter::new(&mut self.stream);
        while let Some(value) = values.pop() {
            let mut chunk = IOChunk::new(self.chunk_size);
            match chunk.serialize(&value) {
                Err(e) => return Err(e),
                Ok(_) => {
                    if let Err(e) = chunk.write(&mut stream) {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Remove element at position `n` and move last vector element
    /// into its empty spot. This method returns None if the vector
    /// did not have at least `n` elements and may return an error
    /// if the underlying was not stream was not seekable, readable or
    /// its content could not be deserialized.
    pub fn swap_remove(&mut self, n: usize) -> IOResult<Option<O>> {
        // Set stream at position.
        let pos = (n * self.chunk_size) as u64;
        if let Err(e) = self.stream.seek(SeekFrom::Start(pos)) {
            return Err(IOError::Seek(e));
        };

        // Save current chunk
        let item = match IOChunk::from_stream(
            self.chunk_size,
            &mut self.stream,
        ) {
            Ok(None) => None,
            Ok(Some(mut item)) => {
                // Deserialize chunk.
                match item.deserialize::<O>() {
                    Ok(o) => Some(o),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        };

        // Get offset of last chunk.
        let end = match self.stream.seek(SeekFrom::End(0)) {
            Ok(p) => p,
            Err(e) => return Err(IOError::Seek(e)),
        };
        // The stream is not large enough to remove any element.
        if end < self.chunk_size as u64 {
            return Ok(None);
        }

        // Move to last chunk offset and copy it into current one.
        // If current chunk is the last we don't need to do this.
        if pos < end - self.chunk_size as u64 {
            // Go to last chunk
            if let Err(e) = self
                .stream
                .seek(SeekFrom::Start(end - self.chunk_size as u64))
            {
                return Err(IOError::Seek(e));
            };

            // Save last chunk
            let last = match IOChunk::from_stream(
                self.chunk_size,
                &mut self.stream,
            ) {
                Ok(c) => c.unwrap(),
                Err(e) => return Err(e),
            };

            // Move to current chunk offset
            if let Err(e) = self.stream.seek(SeekFrom::Start(pos)) {
                return Err(IOError::Seek(e));
            };

            // Write end chunk at current position.
            if let Err(e) = last.write(&mut self.stream) {
                return Err(e);
            };
        }

        // Resize stream to one less chunk.
        //On failure: we could rewrite things how they were and return an
        // error instead.
        self.stream
            .resize(end - self.chunk_size as u64)
            .expect("Failure to resize.");

        Ok(item)
    }

    /// Build an iterator over items of this `IOVec`.
    pub fn iter(&self) -> IOVecIter<O, T> {
        let mut stream = self.stream.clone();
        stream.seek(SeekFrom::Start(0)).unwrap();

        IOVecIter::new(BufReader::new(stream), self.chunk_size)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> IOIter<O, T> {
        IOIter::new(BufReader::new(self.stream), self.chunk_size)
    }

    /// Build an iterator over mutable items of this `IOVec`.
    /// Items modified during iteration will be written back to the
    /// vector underlying stream.
    pub fn iter_mut(&mut self) -> IOVecIterMut<O, T> {
        let mut stream = self.stream.clone();
        stream.seek(SeekFrom::Start(0)).unwrap();
        let pos = stream.clone();
        IOVecIterMut::new(BufReader::new(stream), self.chunk_size, pos)
    }
}

#[cfg(test)]
mod tests {
    use super::IOVec;
    use crate::stream::VecStream;

    #[test]
    fn test_iovec() {
        let mut vec = IOVec::new(VecStream::new(), 32);
        let n = 10;

        // Test append increases length.
        vec.append(&mut vec![[1u8; 4]; 2]).unwrap();
        assert_eq!(vec.len().unwrap(), 2);
        vec.append(&mut vec![[1u8; 4]; n - vec.len().unwrap()])
            .unwrap();
        assert_eq!(vec.len().unwrap(), n);

        // Test appended element can be removed and removal updates len.
        for i in 0..n {
            vec.swap_remove(0).unwrap().unwrap();
            assert_eq!(vec.len().unwrap(), n - i - 1);
        }

        // Once there is nothing to remove, swap remove returns None.
        assert!(vec.swap_remove(0).unwrap().is_none());
        assert!(vec.swap_remove(1).unwrap().is_none());

        // Fill IOVec.
        for i in 0u8..(n as u8) {
            vec.append(&mut vec![[i; 4]]).unwrap();
        }
        assert_eq!(vec.len().unwrap(), n);

        // Make sure every element can be accessed.
        for i in 0..n {
            let item = vec.get(i).unwrap().unwrap();
            assert_eq!(*item, [i as u8; 4]);
        }

        // Make sure every element can be accessed with an iterator.
        for (i, item) in vec.iter().enumerate() {
            assert_eq!(*item, [i as u8; 4]);
        }

        // Make sure every element can be written with an iterator.
        for (i, mut item) in vec.iter_mut().enumerate() {
            assert_eq!(*item, [i as u8; 4]);
            item.copy_from_slice(&[i as u8 + 1u8; 4]);
        }

        // Check writes went through.
        for (i, item) in vec.iter().enumerate() {
            assert_eq!(*item, [i as u8 + 1u8; 4]);
        }

        // Test swap remove.
        assert_eq!(vec.swap_remove(0).unwrap().unwrap(), [1u8; 4]);
        for i in (2u8..11u8).rev() {
            assert_eq!(vec.swap_remove(0).unwrap().unwrap(), [i; 4]);
        }
    }
}

use crate::private::clone::CloneCell;
use crate::private::set::MinSet;
use crate::streams::{IOError, IOResult, Stream};
use crate::{BuildingBlock, Get, GetMut, Ordered, Prefetch};
use lz4::{Decoder, EncoderBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, SeekFrom, Write};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Building block compressing and decompressing it's whole content to
/// perform operations.
///
/// This building blocks stores its elements in a serialized vector,
/// compressed on a stream. Compression/decompression is managed with `lz4`
/// backend. When performing read only operations, the whole content is
/// decompressed. When performing write operations, the whole content
/// is also compressed back to the stream after modification.
///
/// By itself, this building is rather performing poorly both memory and
/// speed wise. It is supposed to be used embedded in another container
/// such as a [`Batch`](struct.Batch.html) or an
/// [`Associative`](struct.Associative.html) container to split the memory
/// footprint into smaller chunks. The
/// [builder](./builder/compression/struct.CompressorBuilder.html) of this
/// building block will embed it into a `Batch` building block.
pub struct Compressor<T: Serialize + DeserializeOwned, S: Stream> {
    stream: S,
    capacity: usize,
    count: CloneCell<usize>,
    unused: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned, S: Stream> Compressor<T, S> {
    pub fn new(stream: S, capacity: usize) -> Self {
        let mut c = Compressor {
            stream,
            capacity,
            count: CloneCell::new(0usize),
            unused: PhantomData,
        };
        *c.count = match c.read() {
            Ok(v) => v.len(),
            Err(_) => 0usize,
        };
        c
    }

    pub fn shallow_copy(&self) -> Self {
        Compressor {
            stream: self.stream.clone(),
            capacity: self.capacity,
            count: self.count.clone(),
            unused: PhantomData,
        }
    }

    pub fn read(&self) -> IOResult<Vec<T>> {
        let mut stream = self.stream.clone();

        // Rewind stream
        if let Err(e) = stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::SeekError(e));
        }

        // Build a decoder of this stream.
        let mut decoder = match Decoder::new(stream) {
            Ok(e) => e,
            Err(e) => return Err(IOError::DecodeError(e)),
        };

        // Decode the whole stream
        let mut bytes: Vec<u8> = Vec::new();

        match decoder.read_to_end(&mut bytes) {
            Ok(_) => {}
            Err(e) => return Err(IOError::DecodeError(e)),
        };

        // Deserialize bytes into an element.
        if !bytes.is_empty() {
            match bincode::deserialize_from(bytes.as_slice()) {
                Ok(t) => Ok(t),
                Err(e) => Err(IOError::DeserializeError(e)),
            }
        } else {
            Ok(Vec::new())
        }
    }

    pub fn write(&mut self, val: &[T]) -> IOResult<()> {
        let mut stream = self.stream.clone();
        let n = val.len();

        // Resize to zero if content is shorter than previous content.
        if let Err(e) = stream.resize(0u64) {
            return Err(IOError::SeekError(e));
        }

        // Rewind stream
        if let Err(e) = stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::SeekError(e));
        }
        // Resize to zero if content is shorter than previous content.
        if let Err(e) = stream.resize(0u64) {
            return Err(IOError::SeekError(e));
        }

        // Serialize element.
        let mut bytes = match bincode::serialize(val) {
            Err(e) => return Err(IOError::SerializeError(e)),
            Ok(v) => v,
        };

        // Compress bytes.
        let mut encoder = match EncoderBuilder::new().build(stream) {
            Ok(e) => e,
            Err(e) => return Err(IOError::EncodeError(e)),
        };

        // Write to stream.
        if let Err(e) = encoder.write(bytes.as_mut_slice()) {
            return Err(IOError::WriteError(e));
        }

        // Finish encoding
        let mut w = match encoder.finish() {
            (_, Err(e)) => {
                return Err(IOError::EncodeError(e));
            }
            (w, Ok(_)) => w,
        };

        // Flush for next operation.
        if let Err(e) = w.flush() {
            return Err(IOError::EncodeError(e));
        }

        *self.count = n;
        Ok(())
    }
}

//------------------------------------------------------------------------//
// BuildingBlock trait
//------------------------------------------------------------------------//

impl<'a, K, V, S> BuildingBlock<'a, K, V> for Compressor<(K, V), S>
where
    K: 'a + Serialize + DeserializeOwned + Eq,
    V: 'a + Serialize + DeserializeOwned + Ord,
    S: Stream,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        *self.count
    }

    fn contains(&self, key: &K) -> bool {
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return false,
            Ok(v) => v,
        };

        v.iter().any(|(k, _)| k == key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        // Read elements into memory.
        let mut v: Vec<(K, V)> = match self.read() {
            Err(_) => return None,
            Ok(v) => v,
        };

        // Look for matching key.
        let i = match v.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i)
            } else {
                None
            }
        }) {
            None => return None,
            Some(i) => i,
        };

        // Remove element from vector and rewrite vector to stream.
        let ret = v.swap_remove(i);
        match self.write(&v) {
            Ok(_) => Some(ret),
            Err(_) => None,
        }
    }

    #[allow(clippy::type_complexity)]
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(n);

        // Read elements into memory (twice).
        let (mut v1, v2): (Vec<(K, V)>, Vec<(K, V)>) =
            match (self.read(), self.read()) {
                (Ok(v1), Ok(v2)) => (v1, v2),
                _ => return out,
            };

        // Look for max values.
        let mut victims = MinSet::new(n);
        for x in v2.into_iter().enumerate().map(|(i, (_, v))| (v, i)) {
            victims.push(x);
        }

        let mut victims: Vec<usize> =
            victims.into_iter().map(|(_, i)| i).collect();
        victims.sort_unstable();

        // Make a vector of max values.
        for i in victims.into_iter().rev() {
            out.push(v1.swap_remove(i));
        }

        // Rewrite vector without popped elements to stream.
        match self.write(&v1) {
            Ok(_) => out,
            Err(_) => Vec::new(),
        }
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        // Read elements into memory.
        let mut v: Vec<(K, V)> = match self.read() {
            Err(_) => return values,
            Ok(v) => v,
        };

        // Insert only fitting elements.
        let n = std::cmp::min(self.capacity - v.len(), values.len());
        let out = values.split_off(n);
        if n > 0 {
            v.append(&mut values);
        }

        // Rewrite vector to stream.
        match self.write(&v) {
            Ok(_) => out,
            Err(_) => panic!("Could not write new elements to Compressor"),
        }
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        // Read elements into memory.
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return Box::new(std::iter::empty()),
            Ok(v) => v,
        };

        if self.stream.resize(0).is_err() {
            return Box::new(std::iter::empty());
        }

        *self.count = 0;
        Box::new(v.into_iter())
    }
}

impl<K, V, S> Ordered<V> for Compressor<(K, V), S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Ord,
    S: Stream,
{
}

//------------------------------------------------------------------------//
// Get Trait
//------------------------------------------------------------------------//

/// Simple struct wrapping a local copy of the value in a
/// `Compressor` building block.
pub struct CompressorCell<V> {
    value: V,
}

impl<V: Ord> Deref for CompressorCell<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Struct wrapping a mutable local copy of the value in a
/// `Compressor` building block.
///
/// The local copy gets written back into the underlying compressed stream
/// when this structure is dropped.
/// The memory footprint of this the total amount of elements in the
/// compressed stream. If you hold several cells of the same compressor,
/// the footprint is multiplied by the amount of cells.
///
/// # Safety:
///
/// On top of the memory footprint, if multiple cells of the same
/// `Compresssor` live and are modified in the same scope, only the last
/// one dropped will be commited back to the compressed stream.
pub struct CompressorMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    stream: Compressor<(K, V), S>,
    elements: Vec<(K, V)>,
    index: usize,
    is_written: bool,
}

impl<K, V, S> Deref for CompressorMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.elements.get(self.index).unwrap().1
    }
}

impl<K, V, S> DerefMut for CompressorMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_written = true;
        &mut self.elements.get_mut(self.index).unwrap().1
    }
}

impl<K, V, S> Drop for CompressorMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    fn drop(&mut self) {
        if !self.is_written {
            return;
        }

        self.stream
            .write(&self.elements)
            .expect("Could not write new elements to Compressor");
    }
}

impl<K, V, S> Get<K, V, CompressorCell<V>> for Compressor<(K, V), S>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize + Ord,
    S: Stream,
{
    unsafe fn get(&self, key: &K) -> Option<CompressorCell<V>> {
        // Read elements into memory.
        match self.read() {
            Err(_) => None,
            Ok(v) => v.into_iter().find_map(|(k, v)| {
                if &k == key {
                    Some(CompressorCell { value: v })
                } else {
                    None
                }
            }),
        }
    }
}

impl<K, V, S> GetMut<K, V, CompressorMutCell<K, V, S>>
    for Compressor<(K, V), S>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize + Ord,
    S: Stream,
{
    unsafe fn get_mut(
        &mut self,
        key: &K,
    ) -> Option<CompressorMutCell<K, V, S>> {
        // Read elements into memory.
        let v = match self.read() {
            Err(_) => return None,
            Ok(v) => v,
        };

        // Find index of matching key.
        let i = match v.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i)
            } else {
                None
            }
        }) {
            None => return None,
            Some(i) => i,
        };

        // Return cell
        Some(CompressorMutCell {
            stream: self.shallow_copy(),
            elements: v,
            index: i,
            is_written: false,
        })
    }
}

//------------------------------------------------------------------------//
// Prefetcher trait
//------------------------------------------------------------------------//

impl<'a, K, V, S> Prefetch<'a, K, V> for Compressor<(K, V), S>
where
    K: 'a + DeserializeOwned + Serialize + Ord,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Stream,
{
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());

        // Read elements into memory.
        let mut v = match self.read() {
            Err(_) => return out,
            Ok(v) => v,
        };

        keys.sort();

        #[allow(clippy::needless_collect)]
        {
            let matches: Vec<usize> = v
                .iter()
                .enumerate()
                .filter_map(|(i, (k, _))| {
                    if keys.binary_search(k).is_ok() {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            for i in matches.into_iter().rev() {
                out.push(v.swap_remove(i));
            }
        }

        // Rewrite vector to stream.
        match self.write(&v) {
            Ok(_) => out,
            Err(_) => {
                panic!("Could not write updated elements to Compressor")
            }
        }
    }
}

//------------------------------------------------------------------------//
// Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Compressor;
    use crate::streams::VecStream;
    use crate::tests::{
        test_building_block, test_get, test_get_mut, test_ordered,
        test_prefetch,
    };

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(Compressor::new(VecStream::new(), i));
        }
    }

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(Compressor::new(VecStream::new(), i));
        }
    }

    #[test]
    fn get() {
        for i in [0usize, 10usize, 100usize] {
            test_get(Compressor::new(VecStream::new(), i));
            test_get_mut(Compressor::new(VecStream::new(), i));
        }
    }

    #[test]
    fn prefetch() {
        for i in [0usize, 10usize, 100usize] {
            test_prefetch(Compressor::new(VecStream::new(), i));
        }
    }
}

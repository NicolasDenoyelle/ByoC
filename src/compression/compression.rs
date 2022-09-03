use crate::internal::SharedPtr;
use crate::stream::{IOError, IOResult, Stream};
use lz4::{Decoder, EncoderBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, SeekFrom, Write};
use std::marker::PhantomData;
use std::ops::DerefMut;

/// Compressed [`BuildingBlock`](trait.BuildingBlock.html) on a byte
/// [stream](utils/stream/trait.Stream.html).
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
/// footprint into smaller chunks and accelerating lookups. The
/// [builder](./builder/compression/struct.CompressedBuilder.html) of this
/// building block will automatically embed it into a `Batch` building block.
///
/// Every operation that uses this container will require to unpack and
/// deserialize the data it contains. If the operation requires mutable access
/// on the container, then, it will also serialize and compress the data back
/// into the underlying [stream](utils/stream/trait.Stream.html). These
/// operations are not optimized to limit memory overhead. The whole stream is
/// unpacked in the main memory. It is up to the overall cache architecture to
/// chunk the data into batches that fit into memory.
///
/// [`Get`](trait.Get.html) trait will return a local copy of the value
/// compressed in the container. [`GetMut`](trait.GetMut.html) trait will wrap
/// this value into a cell that contains a reference to the owning container
/// such that the value can be written back to the compressed stream when the
/// cell is dropped. Since writing back to the stream requires to unpack,
/// deserialize, update, serialize and compress the data, this operation can
/// be costly and with a large memory overhead. To avoid having to unpack and
/// deserialize the stream both to read the value and then to update it, the
/// containing cell also stores a copy of the unpacked and deserialized stream
/// which makes it a potentially large object.
///
/// ## Examples
///
/// ```
/// use byoc::{Compressed, BuildingBlock};
/// use byoc::utils::stream::VecStream;
///
/// let mut compressed = Compressed::new(VecStream::new(), 1);
/// assert_eq!(compressed.push(vec![(0,String::from("first")),
///                                 (1,String::from("second"))]).len(), 1);
/// assert!(compressed.take(&1).is_none());
/// assert_eq!(compressed.take(&0).unwrap().1, "first");
/// ```
///
/// [`Compressed`] container can also be built from a
/// [configuration](config/struct.CompressedConfig.html).
pub struct Compressed<T: Serialize + DeserializeOwned, S: Stream> {
    pub(super) stream: S,
    pub(super) capacity: usize,
    pub(super) count: SharedPtr<usize>,
    pub(super) unused: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned, S: Stream> Compressed<T, S> {
    pub fn new(stream: S, capacity: usize) -> Self {
        let mut c = Compressed {
            stream,
            capacity,
            count: SharedPtr::from(0usize),
            unused: PhantomData,
        };
        *c.count.as_mut().deref_mut() = match c.read() {
            Ok(v) => v.len(),
            Err(_) => 0usize,
        };
        c
    }

    pub(super) fn shallow_copy(&self) -> Self {
        Compressed {
            stream: self.stream.clone(),
            capacity: self.capacity,
            count: self.count.clone(),
            unused: PhantomData,
        }
    }

    pub(super) fn read(&self) -> IOResult<Vec<T>> {
        let mut stream = self.stream.clone();

        // Rewind stream
        if let Err(e) = stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::Seek(e));
        }

        // Build a decoder of this stream.
        let mut decoder = match Decoder::new(stream) {
            Ok(e) => e,
            Err(e) => return Err(IOError::Decode(e)),
        };

        // Decode the whole stream
        let mut bytes: Vec<u8> = Vec::new();

        match decoder.read_to_end(&mut bytes) {
            Ok(_) => {}
            Err(e) => return Err(IOError::Decode(e)),
        };

        // Deserialize bytes into an element.
        if !bytes.is_empty() {
            match bincode::deserialize_from(bytes.as_slice()) {
                Ok(t) => Ok(t),
                Err(e) => Err(IOError::Deserialize(e)),
            }
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) fn write(&mut self, val: &[T]) -> IOResult<()> {
        let n = val.len();

        // Resize to zero if content is shorter than previous content.
        if let Err(e) = self.stream.resize(0u64) {
            return Err(IOError::Seek(e));
        }

        // Rewind stream
        if let Err(e) = self.stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::Seek(e));
        }
        // Resize to zero if content is shorter than previous content.
        if let Err(e) = self.stream.resize(0u64) {
            return Err(IOError::Seek(e));
        }

        // Serialize element.
        let mut bytes = match bincode::serialize(val) {
            Err(e) => return Err(IOError::Serialize(e)),
            Ok(v) => v,
        };

        // Compress bytes.
        let mut encoder =
            match EncoderBuilder::new().build(&mut self.stream) {
                Ok(e) => e,
                Err(e) => return Err(IOError::Encode(e)),
            };

        // Write to stream.
        if let Err(e) = encoder.write(bytes.as_mut_slice()) {
            return Err(IOError::Write(e));
        }

        // Finish encoding
        let w = match encoder.finish() {
            (_, Err(e)) => {
                return Err(IOError::Encode(e));
            }
            (w, Ok(_)) => w,
        };

        // Flush for next operation.
        if let Err(e) = w.flush() {
            return Err(IOError::Encode(e));
        }

        *self.count.as_mut().deref_mut() = n;
        Ok(())
    }
}

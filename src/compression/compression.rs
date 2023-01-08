use crate::stream::{IOError, IOResult, Stream};
use lz4::{Decoder, EncoderBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, SeekFrom, Write};
use std::marker::PhantomData;

/// Compressed `BuildingBlock` on a byte `Stream`.
///
/// This building blocks stores its elements in a serialized vector,
/// compressed on a stream. Compression/decompression is managed with
/// [`lz4`](../lz4/index.html) backend.
/// This [`BuildingBlock`](trait.BuildingBlock.html) is meant to be used
/// embedded in another container such as a [`Batch`](struct.Batch.html) or an
/// [`Associative`](struct.Associative.html) container to split the memory
/// footprint into smaller chunks and accelerating lookups.
///
/// The [builder](./builder/compression/struct.CompressedBuilder.html) of this
/// building block will automatically embed it into a
/// [`Batch`](struct.Batch.html) building block.
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// When performing read only operations, the whole content is
/// decompressed and deserialized into a vector of key/value pairs.
/// From here, operations complexity is the same as an
/// [`Array`](struct.Array.html) container.
/// When performing write operations, the whole content
/// is also serialized then compressed back to the underlying
/// [stream](utils/stream/trait.Stream.html) after modification.
///
/// These operations are not optimized to limit memory overhead.
/// The whole stream is unpacked in the main memory.
/// It is up to the overall cache architecture to chunk the data into batches
/// that fit into memory.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// [`Get`](trait.Get.html) trait will return a local copy of the value
/// compressed in the container. [`GetMut`](trait.GetMut.html) trait will wrap
/// this value into a cell that contains a reference to the owning container
/// such that the value can be written back to the compressed stream when the
/// cell is dropped. Since writing back to the stream requires to unpack,
/// deserialize (read), update, serialize and compress the data (write),
/// this operation can be costly and with a large memory overhead.
/// To avoid having to unpack and
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
/// let mut compressed = Compressed::new(VecStream::new(), 1usize<<10);
/// assert_eq!(compressed.push(vec![(0,String::from("first")),
///                                 (1,String::from("second"))]).len(), 0);
/// assert_eq!(compressed.take(&0).unwrap().1, "first");
/// ```
///
/// [`Compressed`] container can also be built from a
/// [configuration](config/struct.CompressedConfig.html).
pub struct Compressed<T: Serialize + DeserializeOwned, S: Stream> {
    pub(super) stream: S,
    pub(super) capacity: u64,
    pub(super) unused: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned, S: Stream> Compressed<T, S> {
    /// Create a container backed by a compressed
    /// [`Stream`](utils/stream/trait.Stream.html).
    ///
    ///  `capacity`: The container maximum compressed size in bytes.
    pub fn new(stream: S, capacity: usize) -> Self {
        let capacity = capacity as u64;
        Compressed {
            stream,
            capacity,
            unused: PhantomData,
        }
    }

    pub(super) fn shallow_copy(&self) -> Self {
        Compressed {
            stream: self.stream.clone(),
            capacity: self.capacity,
            unused: PhantomData,
        }
    }

    pub(super) fn read_bytes(&self) -> IOResult<Vec<u8>> {
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
            Ok(_) => Ok(bytes),
            Err(e) => Err(IOError::Decode(e)),
        }
    }

    pub(super) fn read(&self) -> IOResult<Vec<T>> {
        let bytes = self.read_bytes()?;

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

    /// Compress and write a slice of values to this stream without
    /// checking if it overflows capacity.
    /// This write overwrite the existing stream.
    pub(super) fn write(&mut self, val: &[T]) -> IOResult<()> {
        // If we attempt to write an empty vector, we can skip a bunch of steps.
        if val.is_empty() {
            if let Err(e) = self.stream.resize(0u64) {
                return Err(IOError::Seek(e));
            }
            return Ok(());
        }

        // Serialize elements.
        let mut bytes = match bincode::serialize(val) {
            Err(e) => return Err(IOError::Serialize(e)),
            Ok(v) => v,
        };

        // Make sure it does not overflow the capacity.
        if bytes.len() > self.capacity as usize {
            return Err(IOError::InvalidSize);
        }

        // Resize stream to zero.
        // If new content is shorter than previous content, we don't get to
        // read invalid elements next time we read the compressed stream.
        if let Err(e) = self.stream.resize(0u64) {
            return Err(IOError::Seek(e));
        }

        // Rewind stream
        if let Err(e) = self.stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::Seek(e));
        }

        // Compress bytes directly to the stream.
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
        Ok(())
    }
}

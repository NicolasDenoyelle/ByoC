use crate::internal::SharedPtr;
use crate::stream::{IOError, IOResult, Stream};
use lz4::{Decoder, EncoderBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, SeekFrom, Write};
use std::marker::PhantomData;
use std::ops::DerefMut;

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
pub struct Compressor<'a, T: Serialize + DeserializeOwned, S: Stream<'a>> {
    pub(super) stream: S,
    pub(super) capacity: usize,
    pub(super) count: SharedPtr<usize>,
    pub(super) unused: PhantomData<&'a T>,
}

impl<'a, T: Serialize + DeserializeOwned, S: Stream<'a>>
    Compressor<'a, T, S>
{
    pub fn new(stream: S, capacity: usize) -> Self {
        let mut c = Compressor {
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
        Compressor {
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

    pub(super) fn write(&mut self, val: &[T]) -> IOResult<()> {
        let n = val.len();

        // Resize to zero if content is shorter than previous content.
        if let Err(e) = self.stream.resize(0u64) {
            return Err(IOError::SeekError(e));
        }

        // Rewind stream
        if let Err(e) = self.stream.seek(SeekFrom::Start(0u64)) {
            return Err(IOError::SeekError(e));
        }
        // Resize to zero if content is shorter than previous content.
        if let Err(e) = self.stream.resize(0u64) {
            return Err(IOError::SeekError(e));
        }

        // Serialize element.
        let mut bytes = match bincode::serialize(val) {
            Err(e) => return Err(IOError::SerializeError(e)),
            Ok(v) => v,
        };

        // Compress bytes.
        let mut encoder =
            match EncoderBuilder::new().build(&mut self.stream) {
                Ok(e) => e,
                Err(e) => return Err(IOError::EncodeError(e)),
            };

        // Write to stream.
        if let Err(e) = encoder.write(bytes.as_mut_slice()) {
            return Err(IOError::WriteError(e));
        }

        // Finish encoding
        let w = match encoder.finish() {
            (_, Err(e)) => {
                return Err(IOError::EncodeError(e));
            }
            (w, Ok(_)) => w,
        };

        // Flush for next operation.
        if let Err(e) = w.flush() {
            return Err(IOError::EncodeError(e));
        }
        drop(w);

        *self.count.as_mut().deref_mut() = n;
        Ok(())
    }
}

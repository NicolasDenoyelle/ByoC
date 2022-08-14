mod stream;
pub use stream::ByteStream;
mod get;
pub use get::{StreamCell, StreamMutCell};
pub(crate) mod builder;
mod building_block;
mod ordered;
mod prefetch;

#[derive(Debug)]
pub enum IOError {
    /// Error returned by call to `seek()` from `std::io::Seek` trait.
    SeekError(std::io::Error),
    /// Error returned by call to `read()` from `std::io::Read` trait.
    ReadError(std::io::Error),
    /// Error returned by call to `write()` from `std::io::Write` trait.
    WriteError(std::io::Error),
    /// Error returned by lz4 encoder builder.
    EncodeError(std::io::Error),
    /// Error returned by lz4 decoder.
    DecodeError(std::io::Error),
    /// Error returned by call to `serialize()` from `bincode::serialize()`.
    SerializeError(bincode::Error),
    /// Error returned by call to `deserialize()` from `bincode::deserialize()`.
    DeserializeError(bincode::Error),
    /// Error related to some size.
    InvalidSizeError,
}

/// Result type of [`byoc::utils::io`](index.html)
/// See [`IOError`](enum.IOError.html).
pub type IOResult<T> = Result<T, IOError>;

/// Resize a byte stream.
///
/// If the stream is extended, the part of the stream beyond
/// current stream size is filled with 0s.
/// If the stream is shrunk, it is truncated from the end of it.
///
/// # Arguments
///
/// * size: The new stream size in bytes.
pub trait Resize {
    fn resize(&mut self, size: u64) -> std::io::Result<()>;
}

/// Facility to spawn stream.
pub trait StreamFactory<S> {
    fn create(&mut self) -> S;
}

/// Combination of traits to work with stream of bytes.
pub trait StreamBase<'a>:
    std::io::Read + std::io::Write + std::io::Seek + Resize
{
    /// The `box_clone()` method must clone into a handle on the
    /// same stream in the same manner as
    /// [File::try_clone()](std::fs::File::try_clone).
    /// The return type is boxed to allow stream to be object safe.
    fn box_clone(&self) -> Box<dyn StreamBase<'a> + 'a>;
}

/// The `clone()` method must clone into a handle on the
/// same stream in the same manner as
/// [File::try_clone()](std::fs::File::try_clone).
/// The return type is boxed to allow stream to be object safe.
pub trait Stream<'a>: StreamBase<'a> + Clone {}

impl<'a> StreamBase<'a> for Box<dyn StreamBase<'a> + 'a> {
    fn box_clone(&self) -> Box<dyn StreamBase<'a> + 'a> {
        (**self).box_clone()
    }
}

impl<'a> Clone for Box<dyn StreamBase<'a> + 'a> {
    fn clone(&self) -> Self {
        (**self).box_clone()
    }
}

impl<'a> Resize for Box<dyn StreamBase<'a> + 'a> {
    fn resize(&mut self, size: u64) -> std::io::Result<()> {
        (**self).resize(size)
    }
}

impl<'a> Stream<'a> for Box<dyn StreamBase<'a> + 'a> {}

mod file_stream;
pub use file_stream::FileStream;
#[cfg(feature = "tempfile")]
pub use file_stream::TempFileStreamFactory;
mod vec_stream;
pub use vec_stream::{VecStream, VecStreamFactory};

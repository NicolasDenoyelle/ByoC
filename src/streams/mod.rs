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
/// If the stream is shrinked, it is truncated from the end of it.
///
/// # Arguments
///
/// * size: The new stream size in bytes.
pub trait Resize {
    fn resize(&mut self, size: u64) -> std::io::Result<()>;
}

/// Facility to spawn streams.
pub trait StreamFactory<S> {
    fn create(&mut self) -> S;
}

/// Combination of traits to work with streams of bytes.
///
/// The clone trait must clone into a resource that represent the
/// same stream in the same manner
/// as [File::try_clone()](std::fs::File::try_clone).
pub trait Stream:
    std::io::Read + std::io::Write + std::io::Seek + Resize + Clone
{
}

mod file_stream;
pub use file_stream::FileStream;
#[cfg(feature = "tempfile")]
pub use file_stream::TempFileStreamFactory;
mod vec_stream;
pub use vec_stream::{VecStream, VecStreamFactory};

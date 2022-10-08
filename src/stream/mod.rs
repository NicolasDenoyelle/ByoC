#[allow(clippy::module_inception)]
mod stream;
pub use stream::ByteStream;
mod get;
pub use get::{StreamCell, StreamMutCell};
pub(crate) mod builder;
mod building_block;
mod file_stream;
mod io_vec;
pub(crate) use io_vec::{IOStructMut, IOVec};
mod error;
pub(crate) use error::{IOError, IOResult};
mod ordered;
pub use file_stream::FileStream;
#[cfg(feature = "tempfile")]
pub use file_stream::TempFileStreamFactory;
mod vec_stream;
pub use vec_stream::{VecStream, VecStreamFactory};
#[cfg(feature = "config")]
pub(crate) mod config;

/// Ability to spawn [`Stream`] instances.
///
/// This trait helps with generating multiple containers based on a stream,
/// for instance, in [`Associative`](../../struct.Associative.html)
/// containers, tests, or builder patterns.
pub trait StreamFactory {
    type Stream: Stream;
    fn create(&mut self) -> Self::Stream;
}

/// Combination of traits to implement containers backed raw `io` bytes.
///
/// This trait represent a stream of raw bytes that is readable (read container
/// items), writable (add or move container items), seekable, and resizeable
/// (remove container items).
///
/// This trait is complemented by the [`Stream`] when implementing
/// container storage based on a raw stream of bytes.
/// The main reason for the existence of this trait on top of the [`Stream`]
/// trait is to be able to create a clonable `<dyn Stream>`.
/// This is required to be able to build a generic
/// stream from a configuration file when the size of the stream cannot be known
/// at compile time.
pub trait StreamBase:
    std::io::Read + std::io::Write + std::io::Seek
{
    /// The `box_clone()` method must clone into a handle on the
    /// same stream in the same manner as
    /// [File::try_clone()](std::fs::File::try_clone).
    /// The return type is boxed to allow stream to be object safe.
    fn box_clone(&self) -> Box<dyn StreamBase>;

    /// Resize a byte stream. (Used to remove or add items to a [`Stream`])
    ///
    /// If the stream is extended, the part of the stream beyond
    /// current stream size is filled with 0s.
    /// If the stream is shrunk, it is truncated from the end of it.
    ///
    /// ## Arguments
    ///
    /// * size: The new stream size in bytes.
    fn resize(&mut self, size: u64) -> std::io::Result<()>;

    fn len(&mut self) -> std::io::Result<u64> {
        self.seek(std::io::SeekFrom::End(0))
    }
}

/// Clonable [`StreamBase`].
///
/// This trait is based of of [`StreamBase`] and adds the clone trait
/// bound. The `clone()` method must clone the underlying stream handle into the
/// same stream in the same manner as [`std::fs::File::try_clone`].
/// This is needed to create in-memory objects from the stream that
/// can be written back in it once they are dropped.
pub trait Stream: StreamBase + Clone {}

impl StreamBase for Box<dyn StreamBase> {
    fn box_clone(&self) -> Box<dyn StreamBase> {
        (**self).box_clone()
    }
    fn resize(&mut self, size: u64) -> std::io::Result<()> {
        (**self).resize(size)
    }
}

impl Clone for Box<dyn StreamBase> {
    fn clone(&self) -> Self {
        (**self).box_clone()
    }
}

impl Stream for Box<dyn StreamBase> {}

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

#[cfg(test)]
mod vstream;
#[cfg(test)]
pub use crate::utils::stream::vstream::{VecStream, VecStreamFactory};

mod file;
pub use crate::utils::stream::file::FileStream;
#[cfg(feature = "tempfile")]
pub use crate::utils::stream::file::TempFileStreamFactory;

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
    #[cfg(feature = "compression")]
    /// Error returned by lz4 encoder builder.
    Encode(std::io::Error),
    #[cfg(feature = "compression")]
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

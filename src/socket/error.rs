use serde::{Deserialize, Serialize};

/// Unrecoverable error forwarded to the [`SocketClient`].
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) enum BincodeErrorKind {
    /// One of
    /// * [`bincode::ErrorKind::InvalidUtf8Encoding(Utf8Error)`],
    /// * [`bincode::ErrorKind::InvalidBoolEncoding(u8)`],
    /// * [`bincode::ErrorKind::InvalidCharEncoding`],
    /// * [`bincode::ErrorKind::InvalidTagEncoding(usize)`].
    InvalidEncoding,
    /// Same as [`bincode::ErrorKind::DeserializeAnyNotSupported`],
    DeserializeAnyNotSupported,
    /// Same as [`bincode::ErrorKind::SizeLimit`],
    SizeLimit,
    /// Same as [`bincode::ErrorKind::SequenceMustHaveLength`],
    SequenceMustHaveLength,
}

/// (Un)Recoverable Error sent to [`SocketClient`] when the [`SocketServer`]
/// encountered an error.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize, Serialize)]
pub(super) enum ResponseError {
    /// [`SocketServer`] received a write back response but there was no
    /// matching data from a `get_mut()` call stored as outgoing data.
    WriteBackWithNoOutgoing,
    /// [`SocketServer`] received a write back response but the element
    /// key to write back did not match the key of the outgoing element.
    InvalidWriteBackKey,
    /// [`SocketServer`] There was an error deserializing data. The error
    /// is due to the received data format.
    BincodeError(BincodeErrorKind),
}

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
    ///
    /// This can happen as a result of break in mutability rules.
    /// If the server is accessed while a `GetMut` value is outgoing and no
    /// `WriteBack` came, it is assumed that the value does not need a write
    /// back because it will never be updated. When a value is not updated, we
    /// can save a WriteBack. So the server does not check whether the `GetMut`
    /// value is actually dropped and keeps going as if it was.
    WriteBackWithNoOutgoing,
    /// [`SocketServer`] There was an error deserializing data. The error
    /// is due to the received data format.
    BincodeError(BincodeErrorKind),
}

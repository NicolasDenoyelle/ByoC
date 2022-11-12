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
    /// [`SocketServer`] There was an error deserializing data. The error
    /// is due to the received data format.
    BincodeError(BincodeErrorKind),
}

/// `SocketServer` error enum.
///
/// Error returned from [`SocketServer`](struct.SocketServer.html) method
/// [`process_request()`](struct.SocketServer.html#method.process_request)
/// method.
#[derive(Debug)]
pub(super) enum SocketServerError {
    /// This is sent back to the client to let them know what happened.
    BincodeError(BincodeErrorKind),
    /// We lost connection to the client.
    BrokenPipe,
    /// Timeout while reading `TcpStream`
    TimedOut,
}

impl From<bincode::Error> for SocketServerError {
    fn from(e: bincode::Error) -> Self {
        match *e {
            bincode::ErrorKind::InvalidUtf8Encoding(_)
            | bincode::ErrorKind::InvalidBoolEncoding(_)
            | bincode::ErrorKind::InvalidCharEncoding
            | bincode::ErrorKind::InvalidTagEncoding(_) => {
                Self::BincodeError(BincodeErrorKind::InvalidEncoding)
            }
            bincode::ErrorKind::DeserializeAnyNotSupported => {
                Self::BincodeError(
                    BincodeErrorKind::DeserializeAnyNotSupported,
                )
            }
            bincode::ErrorKind::SizeLimit => {
                Self::BincodeError(BincodeErrorKind::SizeLimit)
            }
            bincode::ErrorKind::SequenceMustHaveLength => {
                Self::BincodeError(
                    BincodeErrorKind::SequenceMustHaveLength,
                )
            }
            bincode::ErrorKind::Io(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Self::BrokenPipe,
                std::io::ErrorKind::TimedOut => Self::TimedOut,
                std::io::ErrorKind::WouldBlock => Self::TimedOut,
                _unhandled_io_error => {
                    panic!("Unhandled TcpStream IO error: {}", e)
                }
            },
            bincode::ErrorKind::Custom(s) => {
                panic!("Unhandled bincode Custom error {}", s)
            }
        }
    }
}

use super::error::{BincodeErrorKind, ResponseError};
use super::message::{Message, Request, Response};
use crate::{BuildingBlock, Get, GetMut};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug)]
pub enum ServerLoopEvent {
    Ok,
    Timeout,
    /// This error is returned when reading from a `TcpStream` but the amount of
    /// data expected was not there. This may happen when the other end
    /// of the stream closes before we can read any byte from it. It also
    /// may happen because of a faulty peer.
    BrokenPipe,
}

#[derive(Debug)]
enum ServerLoopResult<T> {
    BincodeError(BincodeErrorKind),
    ServerLoopEvent(ServerLoopEvent),
    Ok(T),
}

impl<T> From<bincode::Result<T>> for ServerLoopResult<T> {
    fn from(result: bincode::Result<T>) -> Self {
        match result {
            Ok(t) => Self::Ok(t),
            Err(e) => match *e {
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
                    std::io::ErrorKind::UnexpectedEof => {
                        Self::ServerLoopEvent(ServerLoopEvent::BrokenPipe)
                    }
                    std::io::ErrorKind::WouldBlock => {
                        Self::ServerLoopEvent(ServerLoopEvent::Timeout)
                    }
                    _unhandled_io_error => {
                        panic!("Unhandled TcpStream IO error: {}", e)
                    }
                },
                bincode::ErrorKind::Custom(s) => {
                    panic!("Unhandled bincode Custom error {}", s)
                }
            },
        }
    }
}

/// `BuildingBlock` container remotely accessed through a `TcpStream` by a
/// `SocketClient`.
///
/// The container needs to support
/// [`BuildingBlock`](../traits/trait.BuildingBlock.html),
/// [`Get`](../traits/trait.Get.html) and
/// [`GetMut`](../traits/trait.GetMut.html) traits such that the client can
/// dynamically opt-in or out of some features.
///
/// Keys (`K`) need to be comparable (for equality) to be able to check whether
/// [`GetMut`](../traits/trait.GetMut.html) writes-back are updating the
/// actual outgoing element.
///
/// Values (`V`) need to be clonable such that we can send wrapped values
/// obtained with [`Get`](../traits/trait.Get.html) and
/// [`GetMut`](../traits/trait.GetMut.html) traits without knowing the
/// actual type of the wrapping cell and writeback the updated value.
///
/// Of course, both keys and values need to be serializable and deseriazable.
/// Additional trait bounds can be required on keys and values depending
/// on the `container`
pub struct SocketServer<'a, K, V, C>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V> + Get<K, V> + GetMut<K, V>,
{
    stream: TcpStream,
    pub(super) container: C,
    outgoing: Option<(K, <C as GetMut<K, V>>::Target)>,
    unused_: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, C> SocketServer<'a, K, V, C>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V> + Get<K, V> + GetMut<K, V>,
{
    /// Build a [`SocketServer`] from an initialized `TcpStream` and
    /// `container`.
    fn from_tcp_stream(stream: TcpStream, container: C) -> Self {
        SocketServer {
            stream,
            container,
            outgoing: None,
            unused_: PhantomData,
        }
    }

    /// Set the server timeout waiting on a client request.
    ///
    /// If `dur` is None, the server blocks waiting for a request.
    /// This is the default behaviour.
    pub fn with_timeout(self, dur: Option<Duration>) -> Self {
        self.stream
            .set_read_timeout(dur)
            .expect("Invalid SocketServer timeout");
        self
    }

    /// Build a [`SocketServer`] serving `container` remote accesses to a
    /// [`SocketClient`](struct.SocketClient.html) listening on `address`.
    pub fn new<A: ToSocketAddrs>(
        address: A,
        container: C,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        let (stream, _) = listener.accept()?;
        Ok(Self::from_tcp_stream(stream, container))
    }

    /// Match a message request into a message response containing the
    /// result of accessing the [`SocketServer`] container.
    fn match_response(
        &mut self,
        request: Request<K, V>,
    ) -> Response<K, V> {
        match request {
            Request::Capacity => {
                Response::Capacity(self.container.capacity())
            }
            Request::Size => Response::Size(self.container.size()),
            Request::Contains(k) => {
                Response::Contains(self.container.contains(&k))
            }
            Request::Take(k) => Response::Take(self.container.take(&k)),
            Request::TakeMultiple(mut vec) => Response::TakeMultiple(
                self.container.take_multiple(&mut vec),
            ),
            Request::Pop(size) => Response::Pop(self.container.pop(size)),
            Request::Push(vec) => Response::Push(self.container.push(vec)),
            Request::Flush => {
                Response::Flush(self.container.flush().collect())
            }
            Request::Get(k) => match self.container.get(&k) {
                None => Response::Get(None),
                Some(r) => Response::Get(Some(r.clone())),
            },
            Request::GetMut(k) => match self.container.get_mut(&k) {
                None => Response::GetMut(None),
                Some(r) => {
                    let v = r.clone();
                    self.outgoing.replace((k, r.unwrap()));
                    Response::GetMut(Some(v))
                }
            },
            Request::WriteBack((k, v)) => match self.outgoing.take() {
                Some((key, mut target)) => {
                    if key != k {
                        Response::Error(ResponseError::InvalidWriteBackKey)
                    } else {
                        *target = v;
                        Response::WriteBackAcknowledgment
                    }
                }
                None => {
                    Response::Error(ResponseError::WriteBackWithNoOutgoing)
                }
            },
        }
    }

    /// Do one round of receiving a request, processing it and sending the
    /// response.
    ///
    /// The loop receives requests to access the server container and
    /// answers with the result of the requested container access.
    ///
    /// The function blocks waiting for a
    /// [`SocketClient`](struct.SocketClient.html) request if no timeout is set
    /// on the [`SocketServer`] with
    /// [`with_timeout()`](struct.SocketServer.html#method.with_timeout) method.
    ///
    /// If the function fails to write to the client socket because of a
    /// `std::io::ErrorKind::BrokenPipe` error, the function returns false, i.e
    /// the client is disconnected. On any other error that cannot be
    /// handled, the function panics. On success, true is returned.
    pub fn loop_once(&mut self) -> ServerLoopEvent {
        let request_result = Request::<K, V>::receive(&mut self.stream);
        let response = match ServerLoopResult::from(request_result) {
            ServerLoopResult::BincodeError(e) => {
                Response::Error(ResponseError::BincodeError(e))
            }
            ServerLoopResult::Ok(request) => self.match_response(request),
            ServerLoopResult::ServerLoopEvent(e) => return e,
        };

        let response_result = response.send(&mut self.stream);
        match ServerLoopResult::from(response_result) {
            ServerLoopResult::BincodeError(e) => {
                let response = Response::<K, V>::Error(
                    ResponseError::BincodeError(e),
                );
                let response_result = response.send(&mut self.stream);
                match ServerLoopResult::from(response_result) {
                    ServerLoopResult::BincodeError(_) => {
                        panic!(
                            "SocketServer failed to send bincode error due to a bincode error."
                        );
                    }
                    ServerLoopResult::ServerLoopEvent(
                        ServerLoopEvent::Timeout,
                    ) => panic!(
                        "Unhandled SocketServer timeout sending data."
                    ),
                    ServerLoopResult::ServerLoopEvent(event) => event,
                    ServerLoopResult::Ok(_) => ServerLoopEvent::Ok,
                }
            }
            ServerLoopResult::ServerLoopEvent(
                ServerLoopEvent::Timeout,
            ) => panic!("Unhandled SocketServer timeout sending data."),
            ServerLoopResult::ServerLoopEvent(e) => e,
            ServerLoopResult::Ok(_) => ServerLoopEvent::Ok,
        }
    }
}

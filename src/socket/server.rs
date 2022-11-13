use super::error::{ResponseError, SocketServerError};
use super::message::{Message, Request, Response};
use crate::BuildingBlock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Server connecting a remote `SocketClient` to a local `BuildingBlock`.
///
/// The embedded container needs to support
/// [`BuildingBlock`](../traits/trait.BuildingBlock.html).
/// [`Get`](../traits/trait.Get.html) and
/// [`GetMut`](../traits/trait.GetMut.html) traits are not supported and
/// would very slow to use anyway.
///
/// Key and Value types must of course match on both ends of the
/// `Tcpstream` otherwise deserialization will fail.
/// If request deserialization fails on the server side, the deserialization
/// error is forwarded to the client.
///
/// Both keys and values need to be serializable and deseriazable.
/// Additional trait bounds can be required on keys and values depending
/// on the container on used on the server side.
pub(super) struct SocketServer<'a, K, V, C> {
    stream: TcpStream,
    pub(super) container: C,
    unused: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, C> SocketServer<'a, K, V, C>
where
    K: 'a + DeserializeOwned + Serialize,
    V: 'a + DeserializeOwned + Serialize,
    C: BuildingBlock<'a, K, V>,
{
    /// Build a [`SocketServer`](struct.SocketServer.html) from an
    /// initialized `TcpStream` and `container`.
    fn from_tcp_stream(stream: TcpStream, container: C) -> Self {
        SocketServer {
            stream,
            container,
            unused: PhantomData,
        }
    }

    /// Build a [`SocketServer`](struct.SocketServer.html) using
    /// `container` to serve remote accesses from a
    /// [`SocketClient`](struct.SocketClient.html) listening on `address`.
    ///
    /// This function will block until [`accept()`] method of
    /// [`std::io::TcpListener`] returns, i.e either when a client is connected
    /// or if an error occur.
    pub fn new<A: ToSocketAddrs>(
        address: A,
        container: C,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        let (stream, _) = listener.accept()?;
        Ok(Self::from_tcp_stream(stream, container))
    }

    pub fn with_timeout(self, timeout: Duration) -> Self {
        self.stream
            .set_read_timeout(Some(timeout))
            .expect("Unable to set TcpStream read timeout.");
        self
    }

    /// Match a valid message request into a message response containing the
    /// result of accessing the [`SocketServer`](struct.SocketServer.html)
    /// container.
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
        }
    }

    /// Attempt to accept a connection on the previously connected address.
    ///
    /// This method is intended to be called after
    /// [`process_request()`](struct.SocketServer.html#method.process_request)
    /// returned a
    /// [`SocketServerError::BrokenPipe`](enum.SocketServerError.html#variant.BrokenPipe).
    pub fn try_reconnect(&mut self) -> std::io::Result<()> {
        let address = self.stream.local_addr()?;
        let listener = TcpListener::bind(address)?;
        let (stream, _) = listener.accept()?;
        self.stream = stream;
        Ok(())
    }

    /// Try to receive a request and compute the matching response.
    ///
    /// The method receives a request and deserializes it.
    ///
    /// * If the deserialization works, the request is forwarded to the embedded
    /// container and the output is converted into a response message.
    /// * If a deserialization error occur, the response to the client
    /// will be an error response.
    /// * If the connection is lost, then the corresponding error is returned
    /// and it is up to the caller to handle it.
    /// * Other errors are not handled and the server will panic.
    pub fn process_request(
        &mut self,
    ) -> Result<Response<K, V>, SocketServerError> {
        let request_result = Request::<K, V>::receive(&mut self.stream);
        match request_result.map_err(SocketServerError::from) {
            Ok(request) => Ok(self.match_response(request)),
            Err(SocketServerError::BincodeError(e)) => {
                Ok(Response::Error(ResponseError::BincodeError(e)))
            }
            Err(e) => Err(e),
        }
    }

    /// Try to send a response to the client.
    ///
    /// On success nothing is returned. On error, the error that may be handled
    /// is returned along to with the response to allow retrying the send.
    pub fn process_response(
        &mut self,
        response: Response<K, V>,
    ) -> Result<(), (Response<K, V>, SocketServerError)> {
        let response_result = response.send(&mut self.stream);
        match response_result.map_err(SocketServerError::from) {
            Ok(_) => Ok(()),
            Err(e) => Err((response, e)),
        }
    }
}

use super::error::SocketServerError;
use super::server::SocketServer;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// Struct to pass items to a thread.
/// Items must be consumed by the thread and not transferred across the thread
/// boundaries.
struct CapturedItem<I> {
    item: I,
}
impl<I> CapturedItem<I> {
    pub fn new(item: I) -> Self {
        Self { item }
    }

    pub fn take(self) -> I {
        self.item
    }
}

unsafe impl<I> Sync for CapturedItem<I> {}
unsafe impl<I> Send for CapturedItem<I> {}

/// Helper function to try to reconnect a server.
///
/// If `persistent` is not `Some(0)`,
/// then a reconnect attempt is made. Furthermore, if `persistent` is not `None`,
/// the value inside `persistent` is decremented.
///
/// The function returns whether we successfully attempted to reconnect or not.
fn try_reconnect<'a, K, V, C>(
    server: &mut SocketServer<'a, K, V, C>,
    persistent: &mut Option<usize>,
) -> bool
where
    K: 'a + DeserializeOwned + Serialize,
    V: 'a + DeserializeOwned + Serialize,
    C: BuildingBlock<'a, K, V>,
{
    if match *persistent {
        None => true,     // Yes reconnect always
        Some(0) => false, // No more reconnection
        Some(n) => {
            // Yes reconnect this time.
            persistent.replace(n - 1);
            true
        }
    } {
        server.try_reconnect().is_ok()
    } else {
        false
    }
}

/// Structure used to `spawn` a thread running a `SocketServer`.
///
/// This structure is a helper to create and manage a (private struct)
/// `SocketServer` asynchronously.
///
/// `SocketServer` connects a remote
/// [`SocketClient`](../../struct.SocketClient.html) to a local
/// [`BuildingBlock`](../../trait.BuildingBlock.html).
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
pub struct ServerThreadBuilder<K, V, A, C> {
    // Arguments passed through the thread spawn boundary.
    captured: CapturedItem<(A, C)>,
    // Number of time to restart the server in case of error.
    // Also passed through the thread spawn boundary with copy.
    persistent: Option<usize>,
    unused: PhantomData<(K, V)>,
}

impl<K, V, A, C> ServerThreadBuilder<K, V, A, C>
where
    A: 'static + ToSocketAddrs,
    K: 'static + DeserializeOwned + Serialize,
    V: 'static + DeserializeOwned + Serialize,
    C: 'static + BuildingBlock<'static, K, V>,
{
    /// Create a [`ServerThreadBuilder`].
    ///
    /// The `SocketServer` that will be created in the
    /// [spawned](struct.ServerThreadBuilder.html#method.spawn)
    /// thread will be accepting a connection on `address` and will
    /// serve accesses with `container`.
    /// The input `container` needs to have a static lifetime because it is used
    /// inside of a thread and threads have static lifetimes.
    pub fn new(address: A, container: C) -> Self {
        Self {
            captured: CapturedItem::new((address, container)),
            persistent: Some(0),
            unused: PhantomData,
        }
    }

    /// Set whether to reconnect after receiving an io error
    /// [`std::io::ErrorKind::BrokenPipe`] from the client.
    ///
    /// `num_retry`: If `None`, the server will always try to reconnect to
    /// a client upon disconnection.
    /// If `Some(usize)`, it will try to reconnect up to `usize` times to
    /// a client. If `usize` is `0`, then the server will not try to reconnect
    /// after a lost connection. The default value is `Some(0)`.
    ///
    /// The reconnecting [`SocketClient`](../../struct.SocketClient.html) should
    /// make sure to wait for
    /// this `SocketServer` to start accepting new connections. Otherwise,
    /// the [`SocketClient`](../../struct.SocketClient.html) constructor may return
    /// a [`std::io::Error`] with
    /// error kind [`std::io::ErrorKind::ConnectionRefused`].
    pub fn set_persistent(mut self, num_retry: Option<usize>) -> Self {
        self.persistent = num_retry;
        self
    }

    /// Spawn a thread running a `SocketServer`.
    ///
    /// The `SocketServer` will have this builder address and container.
    ///
    /// This function returns a [`ServerThreadHandle`] that can be used to send
    /// the termination signal to the server thread and join the latter.
    ///
    /// The thread first blocks instanciatiating a
    /// `SocketServer` until a
    /// [`SocketClient`](../../struct.SocketClient.html) connects to it.
    ///
    /// Then, the thread loops with three steps:
    ///
    /// 1. The thread checks if it received the
    /// [stop](struct.ServerThreadHandle.html#method.stop_and_join) signal and
    /// stops if so.
    ///
    /// 2. Then the thread attempts to receive a `Request` message from the
    /// client and processes the matching `Response`.
    /// If the server gets disconnected, the thread may attempt to
    /// reconnect. This can be set with
    /// [`set_persistent()`](struct.ServerThreadBuilder.html#method.set_persistent)
    /// method. If the client request cannot be deserialized in a valid
    /// `Request`, e.g because the client did of have the same types keys and
    /// values, then the error is forwarded to the client in an error
    /// `Response` and handled on the client side.
    ///
    /// 3. Finally the thread attempts to send the `Response` matching the
    /// received `Request` to the client. If the client gets disconnected
    /// while sending the response, one attempt to resend the response will
    /// be done (when the server can reconnect to the client). If reconnection
    /// fails, the thread will stop. If sending fails, the thread will panic.
    ///
    /// The thread may stop because it was ordered to stop, because its client
    /// got disconnected and no further reconnection will be attempted, of
    /// because or an unhandled error. Any unhandled error will result in a
    /// panic of the thread. This panic can only be found about when joining
    /// the thread with
    /// [`stop_and_join()`](struct.ServerThreadHandle.html#method.stop_and_join)
    /// method.
    pub fn spawn(self) -> std::io::Result<ServerThreadHandle<K, V>> {
        let (tx, rx) = channel();

        let ServerThreadBuilder {
            captured,
            mut persistent,
            unused: _,
        } = self;

        let handle = std::thread::Builder::new()
            .name(String::from(std::any::type_name::<Self>()))
            .spawn(move || {
                let (address, container) = captured.take();

                // Create a `SocketServer`. This blocks until a client connects to
                // the server.
                let mut server =
                    SocketServer::new(address, container).unwrap().with_timeout(Duration::from_millis(100));

                // Loop receiving requests and sending responses.
                loop {
                    // Check if we receive the stop message
                    match rx.recv_timeout(Duration::ZERO) {
                        // Yes we did.
                        Ok(_) => break,
                        // No we did not.
                        Err(RecvTimeoutError::Timeout) => {}
                        // The join handle no longer exist.
                        // We assume we should stop the server.
                        Err(RecvTimeoutError::Disconnected) => break,
                    };

                    let response = match server.process_request() {
                        Ok(r) => r,
                        // Try to reconnect
                        Err(SocketServerError::BrokenPipe) => {
                            if try_reconnect(&mut server, &mut persistent)
                            {
                                continue;
                            } else {
                                break;
                            }
                        }
                        Err(SocketServerError::TimedOut) => continue,
			Err(e) => panic!(
                            "Unhandled request processing error: {:?}",
                            e
                        ),
                    };

		    match server.process_response(response) {
			Ok(()) => {},
			Err((response, SocketServerError::BrokenPipe)) => {
                            if !try_reconnect(&mut server, &mut persistent)
                            {
                                break;
                            }
			    if server.process_response(response).is_err() {
				panic!("Failed to send response after reconnection.");
			    }
			}
			Err((_, SocketServerError::TimedOut)) => {
			    panic!("Unexpected timeout writing to TcpStream.");
			}
			Err((_, SocketServerError::BincodeError(_))) => {
			    panic!("Bincode error sending response to client.");
			}
		    }
                }
                drop(server)
            });

        Ok(ServerThreadHandle {
            tx,
            thread_handle: handle?,
            unused: PhantomData,
        })
    }
}

/// Handle returned from
/// [`spawn()`](struct.ServerThreadBuilder.html#method.spawn) method of
/// [`ServerThreadBuilder`] to stop the spawned server thread.
///
/// The handle can be used to stop the server thread and its `SocketServer`.
pub struct ServerThreadHandle<K, V>
where
    K: 'static,
    V: 'static,
{
    tx: Sender<()>,
    thread_handle: JoinHandle<()>,
    unused: PhantomData<(K, V)>,
}

impl<K, V> ServerThreadHandle<K, V>
where
    K: 'static,
    V: 'static,
{
    /// Stop the `SocketServer` in the thread associated with this handle and
    /// join the thread.
    ///
    /// If the server was never connected to a client, this call will hang until
    /// the server has accepted a connection.
    pub fn stop_and_join(self) -> std::thread::Result<()> {
        // Here we signal the thread that we want it to stop.
        // If the receiving end of the channel is disconnected, it means
        // the thread panicked. Either way, whether the thread panicked or
        // received the stop message, we should be able to join the handle.
        self.tx.send(()).unwrap_or(());

        // Join the thread
        self.thread_handle.join()
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::message::{Message, Request, Response};
    use crate::utils::socket::{ServerThreadBuilder, ServerThreadHandle};

    use crate::{Array, BuildingBlock, Concurrent, Sequential};
    use std::{net::TcpStream, time::Duration};

    fn match_response(
        request: &Request<i32, i32>,
        container: &mut Sequential<Array<(i32, i32)>>,
    ) -> Option<Response<i32, i32>> {
        match request {
            Request::Capacity => {
                let result = container.capacity();
                Some(Response::Capacity(result))
            }
            Request::Size => {
                let result = container.size();
                Some(Response::Size(result))
            }
            Request::Contains(k) => {
                let result = container.contains(k);
                Some(Response::Contains(result))
            }
            Request::Take(k) => {
                let result = container.take(k);
                Some(Response::Take(result))
            }
            Request::TakeMultiple(vec) => {
                let result = container.take_multiple(&mut vec.clone());
                Some(Response::TakeMultiple(result))
            }
            Request::Pop(size) => {
                let result = container.pop(*size);
                Some(Response::Pop(result))
            }
            Request::Push(vec) => {
                let result = container.push(vec.clone());
                Some(Response::Push(result))
            }
            Request::Flush => {
                let result = container.flush().collect();
                Some(Response::Flush(result))
            }
        }
    }

    fn make_container_server_client(
        capacity: usize,
        address: &str,
        persistent: Option<usize>,
    ) -> (
        Sequential<Array<(i32, i32)>>,
        TcpStream,
        ServerThreadHandle<i32, i32>,
    ) {
        let container = Sequential::new(Array::new(capacity));
        let server = ServerThreadBuilder::new(
            String::from(address),
            Concurrent::clone(&container),
        )
        .set_persistent(persistent)
        .spawn()
        .expect("SocketServer thread spawn() error.");

        // Wait a bit for the server to start accepting connections.
        std::thread::sleep(Duration::from_millis(50));

        // Connect a client successfully to the server.
        let client = TcpStream::connect(address)
            .expect("Connection to SocketServerThread failed.");

        // Wait a bit for the server to acknowledge connection.
        std::thread::sleep(Duration::from_millis(50));
        (container, client, server)
    }

    #[test]
    fn test_connect() {
        let (_, _, server) =
            make_container_server_client(10, "localhost:6390", Some(0));

        // Make sure the server cleaned itself up and did not panic.
        std::thread::sleep(Duration::from_millis(200));
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_reconnect() {
        let address = "localhost:6391";
        let (_, client, server) =
            make_container_server_client(10, address, Some(1));

        // Cannot reconnect if already connected.
        assert!(TcpStream::connect(address).is_err());

        // Shutdown client and reconnect.
        drop(client);
        let client = loop {
            match TcpStream::connect(address) {
		// Reconnection success.
		Ok(c) => break c,
		Err(e) => match e.kind() {
		    // The server did not register deconnection yet.
		    std::io::ErrorKind::ConnectionRefused => {},
		    // Unexpected error.
		    _ => panic!("First reconnection to SocketServerThread failed: {}.", e),
		}
	    };
        };

        drop(client);
        // Wait for the server to register the deconnection.
        std::thread::sleep(Duration::from_millis(20));
        assert!(TcpStream::connect(address).is_err());

        // Make sure the server cleaned itself up and did not panic.
        server.stop_and_join().unwrap();
    }

    fn test_server_response(
        request: Request<i32, i32>,
        container: &Sequential<Array<(i32, i32)>>,
        client: &mut TcpStream,
    ) {
        let mut container = Clone::clone(container);
        let expected_response = match_response(&request, &mut container);
        request.send(client).unwrap();

        let expected_response = match expected_response {
            None => return,
            Some(r) => r,
        };
        let response = Response::receive(client).unwrap();

        match (response, expected_response) {
            (Response::Capacity(lhs), Response::Capacity(rhs)) => {
                if lhs != rhs {
                    panic!("Capacity response mismatch.")
                }
            }
            (Response::Size(lhs), Response::Size(rhs)) => {
                if lhs != rhs {
                    panic!("Size response mismatch.")
                }
            }
            (Response::Contains(lhs), Response::Contains(rhs)) => {
                if lhs != rhs {
                    panic!("Contains response mismatch.")
                }
            }
            (Response::Error(lhs), Response::Error(rhs)) => {
                if lhs != rhs {
                    panic!("Size response mismatch.")
                }
            }
            (Response::Take(lhs), Response::Take(rhs)) => {
                if lhs != rhs {
                    panic!("Take response mismatch.")
                }
            }
            (Response::TakeMultiple(lhs), Response::TakeMultiple(rhs)) => {
                if lhs != rhs {
                    panic!("TakeMultiple response mismatch")
                }
            }
            (Response::Pop(lhs), Response::Pop(rhs)) => {
                if lhs != rhs {
                    panic!("Pop response mismatch")
                }
            }
            (Response::Push(lhs), Response::Push(rhs)) => {
                if lhs != rhs {
                    panic!("Push response mismatch")
                }
            }
            (Response::Flush(lhs), Response::Flush(rhs)) => {
                if lhs != rhs {
                    panic!("Flush response mismatch")
                }
            }
            r => panic!("Response mismatch: {:?}", r),
        };
    }

    #[test]
    fn test_client_side_response() {
        let (container, mut client, _server) =
            make_container_server_client(10, "localhost:6392", Some(0));

        test_server_response(
            Request::<i32, i32>::Capacity,
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Size,
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Contains(0),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Take(0),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::TakeMultiple(Vec::new()),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Pop(0),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Push(Vec::new()),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::Flush,
            &container,
            &mut client,
        );
    }
}

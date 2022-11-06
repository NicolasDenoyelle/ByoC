use super::{ServerLoopEvent, SocketServer};
use crate::{BuildingBlock, Get, GetMut};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::Cell;
use std::marker::PhantomData;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// Struct to pass items to a thread.
/// Items must be consumed by the thread and not transferred across the thread
/// boundaries.
struct CapturedItem<A, C> {
    address: A,
    container: C,
}
impl<A, C> CapturedItem<A, C> {
    pub fn new(address: A, container: C) -> Self {
        Self { address, container }
    }

    pub fn take(self) -> (A, C) {
        (self.address, self.container)
    }
}

unsafe impl<A, C> Sync for CapturedItem<A, C> {}
unsafe impl<A, C> Send for CapturedItem<A, C> {}

/// Structure used to `spawn` a thread running `SocketServer`.
///
/// This structure is a helper to create and manage a
/// [`SocketServer`](../struct.SocketServer.html) type of
/// [`BuildingBlock`](../trait.BuildingBlock.html) in a different thread
/// than the main thread.
///
/// [`SocketServer`](../../struct.SocketServer.html) instantiation blocks
/// while waiting for a [`SocketClient`](../../struct.SocketClient.html) to
/// connect to it.
pub struct ServerThreadBuilder<K, V, A, C> {
    captured: CapturedItem<A, C>,
    timeout: Option<Duration>,
    unused: PhantomData<(K, V)>,
}

impl<K, V, A, C> ServerThreadBuilder<K, V, A, C>
where
    A: 'static + ToSocketAddrs,
    K: 'static + DeserializeOwned + Serialize + Eq,
    V: 'static + DeserializeOwned + Serialize + Clone,
    C: 'static + BuildingBlock<'static, K, V> + Get<K, V> + GetMut<K, V>,
{
    /// Create a [`ServerThreadBuilder`].
    ///
    /// The [`SocketServer`](../../struct.SocketServer.html) created in the
    /// spawned thread will be accepting a connection on `address` and will
    /// serve accesses with `container`.
    /// The input `container` needs to have a static lifetime because it is used
    /// inside of a thread and threads have static lifetimes.
    ///
    /// If no timeout is set with
    /// [`with_timeout()`](struct.ServerThreadBuilder.html#method.with_timeout)
    /// method, then the
    /// [`spawned`](struct.ServerThreadBuilder.html#method.spawn)
    /// thread will hang waiting for requests from a
    /// [`SocketClient`](../../struct.SocketClient.html) before it can
    /// terminate.
    pub fn new(address: A, container: C) -> Self {
        Self {
            captured: CapturedItem::new(address, container),
            timeout: Some(Duration::from_millis(200)),
            unused: PhantomData,
        }
    }

    /// Attach a socket read timeout [`ServerThreadBuilder`].
    ///
    /// When spawning this builder thread, the method
    /// [`with_timeout()`](../../struct.SocketServer.html#method.with_timeout)
    /// of the created [`SocketServer`](../../struct.SocketServer.html) is
    /// called to set a timeout on reading requests from the associated
    /// [`SocketClient`](../../struct.SocketClient.html)
    /// As a result, if the thread managing the server will be able to stop
    /// on request even if it did not receive any request for a while.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Spawn a thread running a `SocketServer`.
    ///
    /// The [`SocketServer`](../../struct.SocketServer.html) will have the
    /// address container and timeout of the [`ServerThreadBuilder`]
    /// that spawned it.
    ///
    /// The thread first blocks instanciatiating a
    /// [`SocketServer`](../../struct.SocketServer.html) until a
    /// [`SocketClient`](../../struct.SocketClient.html) connects to it.
    ///
    /// Once the [`SocketServer`](../../struct.SocketServer.html) is connected,
    /// the [`ServerThreadHandle`] associated to the thread running the sever
    /// is notified of this event in a non-blocking fashion.
    ///
    /// Then, the thread loops calling
    /// [`loop_once()`](../../struct.SocketServer.html#method.loop_once) method
    /// from the input [`SocketServer`](../../struct.SocketServer.html).
    /// If the method call fails, the thread panics.
    ///
    /// In between each loop, the thread checks if it received a termination
    /// signal and stops looping if so.
    ///
    /// This function returns a [`ServerThreadHandle`] that can be used to send
    /// the termination signal to the server thread and join the latter.
    pub fn spawn(self) -> std::io::Result<ServerThreadHandle> {
        let (handle_tx, rx) = channel();
        let (tx, handle_rx) = channel();

        let ServerThreadBuilder {
            captured,
            timeout,
            unused: _,
        } = self;

        let handle = std::thread::Builder::new()
            .name(String::from(std::any::type_name::<Self>()))
            .spawn(move || {
                let (address, container) = captured.take();

                // Create a `SocketServer`. This blocks until a client connects to
                // the server.
                let mut server = SocketServer::new(address, container)
                    .map(|s| s.with_timeout(timeout))
                    .unwrap();

                // Signal the ServerThreadHandle that the server is connected to
                // a client.
                tx.send(()).unwrap_or(());

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

                    match server.loop_once() {
                        ServerLoopEvent::Ok => {}
                        ServerLoopEvent::BrokenPipe => break,
                        ServerLoopEvent::Timeout => {}
                    }
                }
                drop(server)
            });

        Ok(ServerThreadHandle {
            rx: handle_rx,
            tx: handle_tx,
            server_connected: Cell::new(false),
            thread_handle: handle?,
        })
    }
}

/// Handle returned from
/// [`spawn()`](struct.ServerThreadBuilder.html#method.spawn) method of
/// [`ServerThreadBuilder`] to stop the spawned server thread.
///
/// The handle can be used to check whether the server is connected to a client
/// and ready to operate. It can also stop the server thread and the
/// [`SocketServer`](../../struct.SocketServer.html).
pub struct ServerThreadHandle {
    rx: Receiver<()>,
    tx: Sender<()>,
    server_connected: Cell<bool>,
    thread_handle: JoinHandle<()>,
}

impl ServerThreadHandle {
    /// Stop the [`SocketServer`](../../struct.SocketServer.html) and join the
    /// thread associated with this handle.
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

    /// Check whether the associated [`SocketServer`] has been connected to
    /// a [`SocketClient`](../../struct.SocketClient.html) yet.
    ///
    /// Once the connected state has been reached, this method always returns
    /// true.
    pub fn wait_connection(&self, duration: Duration) -> bool {
        // If we already assessed the server is connected, we don't need to
        // check.
        if self.server_connected.get() {
            return true;
        }

        // If we did not already assessed that the server is connected,
        // we check whether we received the message from it that the
        // `SocketServer` is conneted to a client.
        match self.rx.recv_timeout(duration) {
            Ok(_) => {
                self.server_connected.set(true);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::error::ResponseError;
    use crate::socket::message::{Message, Request, Response};
    use crate::utils::socket::{ServerThreadBuilder, ServerThreadHandle};

    use crate::{
        Array, BuildingBlock, Concurrent, Get, GetMut, Sequential,
    };
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
            Request::Get(k) => {
                let result = container.get(k).map(|r| *r);
                Some(Response::Get(result))
            }
            Request::GetMut(k) => {
                let result = container.get_mut(k).map(|r| *r);
                Some(Response::GetMut(result))
            }
            Request::WriteBack((k, _)) => match container.get(k) {
                Some(_) => None,
                None => Some(Response::Error(
                    ResponseError::WriteBackWithNoOutgoing,
                )),
            },
        }
    }

    fn make_container_server_client(
        capacity: usize,
        address: &str,
    ) -> (Sequential<Array<(i32, i32)>>, TcpStream, ServerThreadHandle)
    {
        let container = Sequential::new(Array::new(capacity));
        let server = ServerThreadBuilder::new(
            String::from(address),
            Concurrent::clone(&container),
        )
        .spawn()
        .expect("SocketServer thread spawn() error.");

        // Wait a bit for the server to accept connections.
        std::thread::sleep(Duration::from_millis(20));

        // Make sure the value of `has_been_connected()` is correct.
        assert!(!server.wait_connection(Duration::ZERO));

        // Connect a client successfully to the server.
        let client = TcpStream::connect(address)
            .expect("Connection to SocketServerThread failed.");

        // Wait a bit for the server to send the connection message
        assert!(server.wait_connection(Duration::from_millis(20)));
        (container, client, server)
    }

    #[test]
    fn test_connect() {
        let (_, _, server) =
            make_container_server_client(10, "localhost:6390");

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
            (
                Response::WriteBackAcknowledgment,
                Response::WriteBackAcknowledgment,
            ) => {}
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
            (Response::Get(lhs), Response::Get(rhs)) => {
                if lhs != rhs {
                    panic!("Get response mismatch")
                }
            }
            (Response::GetMut(lhs), Response::GetMut(rhs)) => {
                if lhs != rhs {
                    panic!("GetMut response mismatch")
                }
            }
            _ => panic!("Response mismatch"),
        };
    }

    #[test]
    fn test_client_side_response() {
        let (container, mut client, _server) =
            make_container_server_client(10, "localhost:6490");

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

        test_server_response(
            Request::<i32, i32>::Get(0),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::GetMut(0),
            &container,
            &mut client,
        );

        test_server_response(
            Request::<i32, i32>::WriteBack((0, 0)),
            &container,
            &mut client,
        );
    }
}

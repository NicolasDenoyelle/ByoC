use super::message::{Message, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

/// `BuildingBlock` running in a remote `SocketServer` and connected through a
/// TcpStream.
///
/// [`SocketClient`] is a local facade to a remote
/// [`BuildingBlock`](traits/trait.BuildingBlock.html) with
/// [`Get`](traits/trait.Get.html) and [`GetMut`](traits/trait.GetMut.html)
/// traits.
pub struct SocketClient {
    pub(super) stream: Arc<Mutex<TcpStream>>,
}

impl SocketClient {
    /// Build a [`SocketClient`] listening on `address`.
    pub fn new<A: ToSocketAddrs>(address: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(SocketClient {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    /// Make send a request to the connected [`SocketServer`] and return
    /// the associated response.
    pub(super) fn process_request<K, V>(
        &self,
        request: Request<K, V>,
    ) -> Response<K, V>
    where
        K: DeserializeOwned + Serialize,
        V: DeserializeOwned + Serialize,
    {
        // We lock the stream for a full round of back and forth
        let mut stream = self.stream.lock().unwrap();

        if let Err(e) = request.send(&mut *stream) {
            panic!("SocketClient failed to send request to SocketServer with error: {}", e);
        };

        match Response::<K, V>::receive(&mut *stream) {
            Ok(response) => response,
            Err(e) => {
                panic!("SocketClient failed to receive request from SocketServer with error: {}", e);
            }
        }
    }
}

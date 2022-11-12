use super::message::{Message, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    marker::PhantomData,
    net::{TcpStream, ToSocketAddrs},
};

/// Make send a request to the connected [`SocketServer`] and return
/// the associated response.
pub(super) fn process_request<K, V>(
    stream: &TcpStream,
    request: Request<K, V>,
) -> Response<K, V>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
{
    let mut stream =
        stream.try_clone().expect("IO Error cloning TcpStream");
    if let Err(e) = request.send(&mut stream) {
        panic!("SocketClient failed to send request to SocketServer with error: {}", e);
    };

    match Response::<K, V>::receive(&mut stream) {
        Ok(response) => response,
        Err(e) => {
            panic!("SocketClient failed to receive request from SocketServer with error: {}", e);
        }
    }
}

/// `BuildingBlock` running in a remote `SocketServer` and connected through a
/// [`std::net::TcpStream`].
///
/// [`SocketClient`] is a local facade to a remote
/// [`BuildingBlock`](traits/trait.BuildingBlock.html) with
/// [`Get`](traits/trait.Get.html) and [`GetMut`](traits/trait.GetMut.html)
/// traits.
pub struct SocketClient<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    pub(super) stream: TcpStream,
    unused: PhantomData<(K, V)>,
}

impl<K, V> SocketClient<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    /// Build a [`SocketClient`] listening on `address`.
    pub fn new<A: ToSocketAddrs>(address: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(SocketClient {
            stream,
            unused: PhantomData,
        })
    }
}

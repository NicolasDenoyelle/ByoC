/// `Stream` struct helpers.
#[cfg(feature = "stream")]
pub mod stream {
    pub use crate::stream::{
        FileStream, Stream, StreamBase, StreamFactory,
        TempFileStreamFactory, VecStream, VecStreamFactory,
    };
}

/// `Profiler` struct helpers.
pub mod profiler {
    pub use crate::profiler::ProfilerOutputKind;
}

/// `Associative` struct helpers.
pub mod associative {
    pub use crate::associative::ExclusiveHasher;
}

pub mod timestamp;

mod lifetime;

/// Objects returned by `Get` and `GetMut` traits implementations.
pub mod get {
    pub use super::lifetime::LifeTimeGuard;
}

#[cfg(feature = "socket")]
/// Server/Client channel between two `BuildingBlock`.
///
/// The [`socket`](index.html) module is the abstraction of a single
/// producer (server), single consumer (client) channel connecting two
/// [`BuildingBlock`](../../trait.BuildingBlock.html) over a
/// [`std::net::TcpStream`].
///
/// It is enabled using the `socket` build feature.
/// It is composed of three main components:
/// 1. `SocketServer` a private structure managing remote requests to access
/// a local [`BuildingBlock`](../../trait.BuildingBlock.html) container.
/// 2. [`SocketClient`](../../struct.SocketClient.html) the entity sending
/// remote call requests to a remote
/// [`BuildingBlock`](../../trait.BuildingBlock.html) container held in a
/// `SocketServer`.
/// 3. [`ServerThreadBuilder`](struct.ServerThreadBuilder.html) and
/// [`ServerThreadHandle`](struct.ServerThreadHandle.html) to instantiate
/// and manage a `SocketServer` in a non-blocking way.
///
/// ## Example
///
/// In below example we setup a client
/// [`BuildingBlock`](../../trait.BuildingBlock.html) which methods are
/// forwarded to a `SocketServer` backed by an
/// [`Array`](../../struct.Array.html) of 10 elements.
///
/// ```
/// use byoc::{BuildingBlock, Array, SocketClient};
/// use byoc::utils::socket::ServerThreadBuilder;
///
/// // This is the address on which we connect the server and client.
/// let address = "localhost:6590";
///
/// // We create a server backed by an `Array` of up to 10 elements.
/// let container = Array::<(i32,i32)>::new(10usize);
/// let server = ServerThreadBuilder::new(address, container)
///     .spawn()
///     .unwrap();
///
/// // We wait a little for the server to start accepting new connections.
/// std::thread::sleep(std::time::Duration::from_millis(50));
///
/// // Next, we can create a client to connect to the server.
/// let mut client = SocketClient::<i32,i32>::new(address).unwrap();
///
/// // Now we can perform accesses on the client that are serviced from the
/// // server.
/// assert_eq!(client.capacity(), 10usize);
/// assert!(!client.contains(&0));
/// client.push(vec![(0, 0)]);
/// assert!(client.contains(&0));
///
/// // Finally we cleanup the server.
/// server.stop_and_join().unwrap();
/// ```
pub mod socket {
    pub use crate::socket::{ServerThreadBuilder, ServerThreadHandle};
}

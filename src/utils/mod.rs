/// `Stream` struct helpers.
#[cfg(feature = "stream")]
pub mod stream {
    pub use crate::stream::{
        FileStream, Stream, StreamBase, StreamFactory,
        TempFileStreamFactory, VecStream, VecStreamFactory,
    };
}

/// Decoration wrappers for values and traits for `Decorator` `BuildingBlock`.
///
/// This module is a companion module of
/// [`Decorator`](../../struct.Decorator.html) container. The module provides
/// two traits: [`Decoration`](trait.Decoration.html) and
/// [`DecorationFactory`](trait.DecorationFactory.html).
///
/// [`Decoration`](trait.Decoration.html)s are structures used to wrap values
/// inserted in [`Decorator`](../struct.Decorator.html) containers.
/// They may provide values with an
/// implementation of a trait that affect the decorated container.
/// They are instantiated through a
/// [`DecorationFactory`](trait.DecorationFactory.html) that may customize
/// each individual [`Decoration`](trait.Decoration.html)s.
///
/// ### Examples
///
/// In below example we show how to add a [`Fifo`](struct.Fifo.html) eviction
/// policy to an
/// [`Array`](../../struct.Array.html) container. `Array` container is a
/// container that evicts its largest elements when an eviction is needed.
/// Since `Fifo` [`DecorationFactory`](trait.DecorationFactory.html)
/// orders value in the order they are
/// wrapped (i.e inserted in the container), the first values evicted will be
/// the first values inserted.
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Decorator};
/// use byoc::utils::decorator::Fifo;
///
/// let mut c = Decorator::new(Array::new(3), Fifo::new());
/// assert_eq!(c.push(vec![("item1",1u16), ("item2",2u16), ("item0",0u16)])
///             .len(), 0);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
/// ```
pub mod decorator {
    pub use crate::decorator::{
        Decoration, DecorationFactory, Fifo, Lrfu, Lru,
    };
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

//-----------------------------------------------------------------------------
// Private utils.
//-----------------------------------------------------------------------------

/// Math utils
pub(crate) mod math;
/// Utils for obtaining multiple writable shallow copies of a resource.
mod shared_ptr;
pub(crate) use shared_ptr::SharedPtr;
#[cfg(feature = "stream")]
/// Utils for extracting k min elements.
pub(crate) mod kmin;
/// Library custom read/write lock.
pub(crate) mod lock;

/// Size computation utils.
pub(crate) mod size;

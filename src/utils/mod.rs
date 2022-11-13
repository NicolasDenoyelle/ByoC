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

/// Eviction policies and related traits and modules.
///
/// Containers eviction is implemented by the
/// [`building blocks`](../trait.BuildingBlock.html#implementors)
/// themselves when calling the
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop) method.
/// The when sthe container implements the
/// [`Ordered`](trait.Ordered.html) trait, the pop
/// method will try to take out the elements with the highest value.
///
/// The [`Policy`](../struct.Policy.html) container is wrapper around
/// such a container (although the container does not need to carry the
/// [`Ordered`](trait.Ordered.html) trait bound) that will wrap values into
/// a [`Reference`](trait.Reference.html)
/// cell ordering values in the container with a specific policy.
///
/// This is a generic, but potentially inefficient, way to customize the
/// eviction policy on a wide range of containers.
///
/// [`Lrfu`](struct.Lrfu.html) and [`Lru`](struct.Lru.html) policies will
/// change the order of elements
/// in the container when they are accessed from within the container
/// using [`Get`](../trait.Get.html) and [`GetMut`](../trait.GetMut.html)
/// traits. This is potentially dangerous! Indeed, if the container relies
/// on the order of its elements (for instance it uses a
/// [`std::collections::BTreeSet`]), then
/// accessing elements inside the container will make things dicey.
/// If the container does not
/// implement the [`Ordered`](trait.Ordered.html) trait bound, it is probably
/// a bad idea to use
/// on of these policies.
///
/// ### Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Policy};
/// use byoc::utils::policy::Fifo;
///
/// let mut c = Policy::new(Array::new(3), Fifo::new());
/// assert_eq!(c.push(vec![("item1",1u16), ("item2",2u16), ("item0",0u16)])
///             .len(), 0);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
/// ```
pub mod policy {
    pub use crate::policy::{
        timestamp, Fifo, Lrfu, Lru, Ordered, Reference, ReferenceFactory,
    };
}

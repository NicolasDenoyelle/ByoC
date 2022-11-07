use super::{SocketClient, SocketServer};
use crate::builder::Build;
use crate::BuildingBlock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::net::ToSocketAddrs;

/// `SocketClient` builder.
///
/// Builder pattern to build `SocketClient` listening on a socket address.
pub struct SocketClientBuilder<A: ToSocketAddrs> {
    address: A,
}

impl<A: ToSocketAddrs> SocketClientBuilder<A> {
    /// Create a [`SocketClientBuilder`] listening on `address`.
    pub fn new(address: A) -> Self {
        SocketClientBuilder { address }
    }
}

impl<A: ToSocketAddrs + Clone> Clone for SocketClientBuilder<A> {
    fn clone(&self) -> Self {
        SocketClientBuilder {
            address: self.address.clone(),
        }
    }
}

impl<A: ToSocketAddrs> Build<SocketClient> for SocketClientBuilder<A> {
    fn build(self) -> SocketClient {
        SocketClient::new(self.address).unwrap()
    }
}

/// `SocketServer` builder.
///
/// Builder pattern to build [`SocketServer`](../../struct.SocketServer.html)
/// accepting one connection on a socket address and serving accesses with a
/// container built from a builder pattern.
///
/// This builder is obtained from using
/// [`ServerBuild`](../trait.ServerBuild.html) trait.
pub struct SocketServerBuilder<'a, K, V, A, B, C>
where
    A: ToSocketAddrs,
    B: Build<C>,
{
    address: A,
    container_builder: B,
    unused: PhantomData<&'a (K, V, C)>,
}

impl<'a, K, V, A, B, C> SocketServerBuilder<'a, K, V, A, B, C>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    A: ToSocketAddrs,
    B: Build<C>,
    C: BuildingBlock<'a, K, V>,
{
    /// Build a `SocketServerBuilder` accepting one connection on
    /// `address` and serving accesses with a container built from a
    /// `container_builder`.
    pub fn new(address: A, container_builder: B) -> Self {
        SocketServerBuilder {
            address,
            container_builder,
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, A, B, C> Clone for SocketServerBuilder<'a, K, V, A, B, C>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    A: ToSocketAddrs + Clone,
    B: Build<C> + Clone,
    C: BuildingBlock<'a, K, V>,
{
    fn clone(&self) -> Self {
        SocketServerBuilder {
            address: self.address.clone(),
            container_builder: self.container_builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, A, B, C> Build<SocketServer<'a, K, V, C>>
    for SocketServerBuilder<'a, K, V, A, B, C>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    A: ToSocketAddrs,
    B: Build<C>,
    C: BuildingBlock<'a, K, V>,
{
    fn build(self) -> SocketServer<'a, K, V, C> {
        SocketServer::new(self.address, self.container_builder.build())
            .unwrap()
    }
}

/// Make a container builder into a `SocketServer` for a remote `SocketClient`
/// container.
pub trait ServerBuild<'a, K, V, C>: Build<C> + Sized
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V>,
{
    /// Make this builder into a server builder accepting one  remote
    /// [`SocketClient`](../struct.SocketClient.html) connection at `address`.
    fn accept<A: ToSocketAddrs>(
        self,
        address: A,
    ) -> SocketServerBuilder<'a, K, V, A, Self, C> {
        SocketServerBuilder::new(address, self)
    }
}

impl<'a, K, V, C, B> ServerBuild<'a, K, V, C> for B
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V>,
    B: Build<C> + Sized,
{
}

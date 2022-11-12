use super::SocketClient;
use super::{ServerThreadBuilder, ServerThreadHandle};
use crate::builder::Build;
use crate::BuildingBlock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::net::ToSocketAddrs;

/// `SocketClient` builder.
///
/// Builder pattern to build a [`SocketClient`](../struct.SocketClient.html)
/// listening on a socket address.
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

impl<A, K, V> Build<std::io::Result<SocketClient<K, V>>>
    for SocketClientBuilder<A>
where
    A: ToSocketAddrs,
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    fn build(self) -> std::io::Result<SocketClient<K, V>> {
        SocketClient::new(self.address)
    }
}

/// `SocketServer` builder.
///
/// Builder pattern to build `SocketServer` managed in a separate thread
/// by a
/// [`ServerThreadBuilder`](../utils/socket/struct.ServerThreadBuilder.html).
/// The server accepts one connection on a socket address and serves
/// accesses with a container built from a builder pattern.
///
/// This builder is obtained from using
/// [`ServerBuild`](trait.ServerBuild.html) trait.
pub struct SocketServerBuilder<K, V, A, C, B>
where
    A: ToSocketAddrs,
    B: Build<C>,
{
    address: A,
    container_builder: B,
    unused: PhantomData<(K, V, C)>,
}

impl<K, V, A, C, B> SocketServerBuilder<K, V, A, C, B>
where
    A: ToSocketAddrs,
    B: Build<C>,
{
    /// Build a [`SocketServerBuilder`] accepting one connection on
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

impl<K, V, A, C, B> Clone for SocketServerBuilder<K, V, A, C, B>
where
    A: ToSocketAddrs + Clone,
    B: Build<C> + Clone,
{
    fn clone(&self) -> Self {
        SocketServerBuilder {
            address: self.address.clone(),
            container_builder: self.container_builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<K, V, A, C, B> Build<std::io::Result<ServerThreadHandle<K, V>>>
    for SocketServerBuilder<K, V, A, C, B>
where
    A: 'static + ToSocketAddrs + Clone,
    B: Build<C> + Clone,
    K: 'static + DeserializeOwned + Serialize + Eq,
    V: 'static + DeserializeOwned + Serialize + Clone,
    C: 'static + BuildingBlock<'static, K, V>,
{
    fn build(self) -> std::io::Result<ServerThreadHandle<K, V>> {
        let builder = ServerThreadBuilder::<K, V, A, C>::new(
            self.address,
            self.container_builder.build(),
        );
        builder.spawn()
    }
}

/// Make a container builder into a [`SocketServerBuilder`].
pub trait ServerBuild<'a, K, V, C>: Build<C> + Sized
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V>,
{
    /// Make this builder into a server builder accepting one remote
    /// [`SocketClient`](../struct.SocketClient.html) connection at `address`.
    fn accept<A: ToSocketAddrs>(
        self,
        address: A,
    ) -> SocketServerBuilder<K, V, A, C, Self> {
        SocketServerBuilder::new(address, self)
    }
}

impl<'a, K, V, C, B> ServerBuild<'a, K, V, C> for B
where
    B: Build<C> + Sized,
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Clone,
    C: BuildingBlock<'a, K, V>,
{
}

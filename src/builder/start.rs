use crate::builder::builders::{ArrayBuilder, BTreeBuilder};
use crate::builder::traits::{Array, BTree};

#[cfg(feature = "stream")]
use crate::builder::builders::ByteStreamBuilder;
#[cfg(feature = "stream")]
use crate::builder::traits::ByteStream;
#[cfg(feature = "stream")]
use crate::container::stream::{Stream, StreamFactory};
#[cfg(feature = "stream")]
use serde::{de::DeserializeOwned, Serialize};

pub struct Start {}

impl Clone for Start {
    fn clone(&self) -> Self {
        Start {}
    }
}

impl<T> Array<T, ArrayBuilder<T>> for Start {
    fn array(self, capacity: usize) -> ArrayBuilder<T> {
        ArrayBuilder::new(capacity)
    }
}

impl<K: Copy + Ord, V: Ord> BTree<K, V, BTreeBuilder<K, V>> for Start {
    fn btree(self, capacity: usize) -> BTreeBuilder<K, V> {
        BTreeBuilder::new(capacity)
    }
}

#[cfg(feature = "stream")]
impl<T, S, F> ByteStream<T, S, F, ByteStreamBuilder<T, S, F>> for Start
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    fn byte_stream(
        self,
        factory: F,
        capacity: usize,
    ) -> ByteStreamBuilder<T, S, F> {
        ByteStreamBuilder::new(factory, capacity)
    }
}

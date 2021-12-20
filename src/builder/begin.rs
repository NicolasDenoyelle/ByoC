#[cfg(feature = "compression")]
use crate::builder::builders::CompressorBuilder;
#[cfg(feature = "stream")]
use crate::builder::builders::StreamBuilder;
use crate::builder::builders::{ArrayBuilder, BTreeBuilder};
#[cfg(feature = "stream")]
use crate::streams::{Stream, StreamFactory};
#[cfg(feature = "stream")]
use serde::{de::DeserializeOwned, Serialize};

/// Begin a container builder chain.
///
/// This builder can be consumed to produce a the first component
/// of a [building block](../../trait.BuildingBlock.html) chain.
/// In order to start the chain, you have to call one of the
/// struct methods.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::builder::Begin;
///
/// // Build a multi-layer concurrent cache where the first layer stores
/// // up to two elements in an `Array` type container and the second layer
/// // stores up to four elements into a `BTree` type container.
/// let mut container = Begin::array(2).multilevel(Begin::btree(4)).into_sequential().build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct Begin {}

impl Begin {
    pub fn array<T>(capacity: usize) -> ArrayBuilder<T> {
        ArrayBuilder::new(capacity)
    }

    pub fn btree<K: Copy + Ord, V: Ord>(
        capacity: usize,
    ) -> BTreeBuilder<K, V> {
        BTreeBuilder::new(capacity)
    }

    #[cfg(feature = "stream")]
    pub fn byte_stream<
        T: DeserializeOwned + Serialize,
        S: Stream,
        F: StreamFactory<S> + Clone,
    >(
        factory: F,
        capacity: usize,
    ) -> StreamBuilder<T, S, F> {
        StreamBuilder::new(factory, capacity)
    }

    #[cfg(feature = "compression")]
    pub fn compressor<
        T: DeserializeOwned + Serialize,
        S: Stream,
        F: StreamFactory<S> + Clone,
    >(
        factory: F,
        num_batch: usize,
        batch_capacity: usize,
    ) -> CompressorBuilder<T, S, F> {
        CompressorBuilder::new(num_batch, batch_capacity, factory)
    }
}

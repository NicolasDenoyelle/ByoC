#[cfg(feature = "compression")]
use crate::builder::builders::CompressedBuilder;
#[cfg(feature = "stream")]
use crate::builder::builders::StreamBuilder;
use crate::builder::builders::{ArrayBuilder, BTreeBuilder};
#[cfg(feature = "stream")]
use crate::stream::StreamFactory;
#[cfg(feature = "stream")]
use serde::{de::DeserializeOwned, Serialize};

/// Entry point to build a container from builder pattern chain.
///
/// This builder can be consumed to produce a the first component
/// of a [building block](../../trait.BuildingBlock.html) chain.
/// In order to start the chain, you have to call one of the
/// struct methods.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::builder::Builder;
///
/// // Build a multi-layer concurrent cache where the first layer stores
/// // up to two elements in an `Array` type container and the second layer
/// // stores up to four elements into a `BTree` type container.
/// let mut container = Builder::array(2)
///     .exclusive(Builder::btree(4))
///     .into_sequential()
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct Builder {}

impl Builder {
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
        F: StreamFactory + Clone,
    >(
        factory: F,
        capacity: usize,
    ) -> StreamBuilder<T, F> {
        StreamBuilder::new(factory, capacity)
    }

    #[cfg(feature = "compression")]
    pub fn compressed<
        T: DeserializeOwned + Serialize,
        F: StreamFactory + Clone,
    >(
        factory: F,
        num_batch: usize,
        batch_capacity: usize,
    ) -> CompressedBuilder<T, F> {
        CompressedBuilder::new(num_batch, batch_capacity, factory)
    }
}

use crate::builder::builders::{ArrayBuilder, BTreeBuilder};

#[cfg(feature = "stream")]
use crate::builder::builders::ByteStreamBuilder;
#[cfg(feature = "stream")]
use crate::container::stream::{Stream, StreamFactory};
#[cfg(feature = "stream")]
use serde::{de::DeserializeOwned, Serialize};

/// Begin a container
/// [builder](../traits/trait.Builder.html) chain.
///
/// This builder can be consumed to produce a the first component
/// of a [building block](../../trait.BuildingBlock.html) chain.
/// In order to start the chain, you have to call one of the
/// struct methods.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::Begin;
///
/// // Build a multi-layer concurrent cache where the first layer stores
/// // up to two elements in an `Array` type container and the second layer
/// // stores up to four elements into a `BTree` type container.
/// let mut container = Begin::array(2).forward(Begin::btree(4)).into_sequential().build();
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
    ) -> ByteStreamBuilder<T, S, F> {
        ByteStreamBuilder::new(factory, capacity)
    }
}

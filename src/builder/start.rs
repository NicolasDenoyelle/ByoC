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

/// Empty struct used to start a container
/// [builder](../traits/trait.Builder.html) chain.
///
/// This builder can be consumed to produce a the first component
/// of a [building block](../../trait.BuildingBlock.html) chain.
/// In order to start the chain, you can call one of the methods:
/// [`array()`](../traits/trait.Array.html#tymethod.array),
/// [`btree()`](../traits/trait.BTree.html#tymethod.btree),
/// [`byte_stream()`](../traits/trait.ByteStream.html#tymethod.byte_stream),
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::Start;
///
/// // Build a multi-layer concurrent cache where the first layer stores
/// // up to two elements in an `Array` type container and the second layer
/// // stores up to four elements into a `BTree` type container.
/// let mut container = Start{}.array(2).forward().btree(4).into_sequential().build();
/// container.push(vec![(1, 2)]);
/// ```
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

#![doc = include_str!("../README.md")]

//-------------------------------------------------------------------------
// Traits
//-------------------------------------------------------------------------

mod building_block;
pub use building_block::BuildingBlock;
mod get;
pub use get::{Get, GetMut};
mod prefetch;
pub use prefetch::Prefetch;
mod concurrent;
pub use concurrent::Concurrent;
mod ordered;
pub use ordered::Ordered;

/// Helpers to easily build complex building block chain.
///
/// Builder module provides the tool ease the process of building
/// a complex building block chain.
///
/// Consider the following key/value store architecture:   
/// The container is made of two layers, where the first layer
/// uses an [Array](../struct.Array.html)
/// [building block](../trait.BuildingBlock.html) with a capacity
/// of 10000 elements. The second layer uses a
/// [BTree](../struct.BTree.html) building block with
/// a capacity of 1000000 elements. The two containers are connected
/// with a [Multilevel](../struct.Multilevel.html) connector.
/// We want the [most recently used](../policy/struct.LRU.html) elements
/// to stay in the first layer, and we want to be able to access the
/// container [concurrently](../trait.Concurrent.html).
///
/// Without the builder pattern, such container would be built as follow:
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, BTree, Multilevel, Sequential, Policy};
/// use byoc::policy::{LRU, timestamp::Clock};
///
/// let array = Array::new(10000);
/// let btree = BTree::new(1000000);
/// let multilevel = Multilevel::new(array, btree);
/// let policy = Policy::new(multilevel, LRU::<Clock>::new());
/// let mut container = Sequential::new(policy);
/// container.push(vec![(1,2)]);
/// ```
///
/// With a builder pattern, the same code becomes:
/// ```
/// use byoc::BuildingBlock;
/// use byoc::policy::{LRU, timestamp::Clock};
/// use byoc::builder::traits::*;
/// use byoc::builder::Begin;
///
/// let mut container = Begin::array(10000).multilevel(Begin::btree(1000000)).with_policy(LRU::<Clock>::new()).into_sequential().build();
/// container.push(vec![(1,2)]);
/// ```
pub mod builder;

mod array;
pub use array::{Array, ArrayCell, ArrayMutCell};
mod associative;
pub use associative::{Associative, MultisetHasher};
mod batch;
pub use batch::Batch;
mod btree;
pub use btree::{BTree, BTreeCell};
#[cfg(feature = "compression")]
mod compression;
#[cfg(feature = "compression")]
pub use compression::{Compressor, CompressorCell, CompressorMutCell};
mod multilevel;
pub use multilevel::{Multilevel, MultilevelCell};
mod profiler;
pub use profiler::{Profiler, ProfilerOutputKind};
/// Policies implementation for `Policy` building block.
pub mod policy;
pub use policy::policy::{Policy, PolicyCell};
mod sequential;
pub use sequential::{Sequential, SequentialCell};
#[cfg(feature = "stream")]
pub mod stream;
#[cfg(feature = "stream")]
pub use stream::{ByteStream as Stream, StreamCell, StreamMutCell};

/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod internal;

/// Public test module available at test time.
///
/// This module tests the expected behavior of
/// [`BuildinlBlock`](trait.BuildingBlock.html) and
/// [`Get`](trait.Get.html) traits with
/// `test_building_block()` and `test_get()`.
#[cfg(test)]
mod tests;

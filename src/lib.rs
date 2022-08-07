#![doc = include_str!("../README.md")]

use std::ops::{Deref, DerefMut};
/// This is the main abstraction: the Building block interface for
/// key/value storage.
///
/// `BuildingBlock` trait defines the primitives to build a
/// data processing pipeline implementing a key/value data container.
/// The interface is made such that `BuildingBlock` implementers can be
/// assembled in a pipeline fashion to build a container that will meet
/// features and performance requirement of users key/value access
/// workloads.
///
/// A typical key/value container implementation could be a cache
/// with multiple layers of increasing size and decreasing performance,
/// with an eviction policy such that most accessed data live in the
/// fastest layer.
///
/// See
/// [`BuildingBlock` implementors](trait.BuildingBlock.html#implementors)
/// for more details on structuring building blocks together.
pub trait BuildingBlock<'a, K: 'a, V: 'a> {
    /// Get the maximum number of elements fitting in the container.
    /// The actual number may be smaller depending on the implementation.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a matchig key.
    fn contains(&self, key: &K) -> bool;

    /// Take the matching key/value pair out of the container.
    fn take(&mut self, key: &K) -> Option<(K, V)>;

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// The eviction policy deciding which elements are popped out is
    /// implementation defined.
    /// Implementations also implementing the marker trait
    /// [`Ordered`](policies/trait.Ordered.html) will guarantee the eviction
    /// of elements with the largest value. Usually, such building block
    /// are meant to be wrapped into a
    /// [`Policy`](struct.Policy.html)
    /// `BuildingBlock` to define the eviction policy.
    fn pop(&mut self, n: usize) -> Vec<(K, V)>;

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    /// The trait does not make a contract on what is returned.
    /// It could be for instance the values not fitting in the container or
    /// some values from the container depending on trade-offs
    /// or desired behavior.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a>;
}

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
pub trait Get<K, V, U>
where
    U: Deref<Target = V>,
{
    /// Get a read-only smart pointer to a value inside the container.
    ///
    /// # Safety
    ///
    /// At this time, it does not seam feasible to return a trait object
    /// with the same lifetime as the function call. Therefore, any lifetime
    /// inference on the returned structure would require it to have the
    /// same lifetime as the building block instance which would for
    /// instance prevent to call this trait method in a loop. As a result,
    /// this trait implementation maybe `unsafe`, because the returned
    /// guard lifetime may outlive the borrowing lifetime of the container
    /// where the inner value originates from.
    unsafe fn get(&self, key: &K) -> Option<U>;
}
/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to a mutable values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// mutable values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
/// This trait is separated from [`Get`](trait.Get.html) because
/// some containers ([BTree](struct.BTree.html)) have to
/// be mutated when they are accessed, hence they can implement `get_mut()`
/// but not `get()`. These two traits may also require different trait
/// bounds because, for instance int the former the value can be moved
/// from a building block not implementing `GetMut` to one implementing
/// it and returning the value from there
/// (See [`Multilevel`](struct.Multilevel.html)).
pub trait GetMut<K, V, W>
where
    W: Deref<Target = V> + DerefMut,
{
    /// Get a smart pointer to a mutable value inside the container.
    ///
    /// # Safety
    ///
    /// At this time, it does not seam feasible to return a trait object
    /// with the same lifetime as the function call. Therefore, any lifetime
    /// inference on the returned structure would require it to have the
    /// same lifetime as the building block instance which would for
    /// instance prevent to call this trait method in a loop. As a result,
    /// this trait implementation maybe `unsafe`, because the returned
    /// guard lifetime may outlive the borrowing lifetime of the container
    /// where the inner value originates from.
    unsafe fn get_mut(&mut self, key: &K) -> Option<W>;
}

/// This is a marker trait for [`BuildingBlock`](trait.BuildingBlock.html).
/// When this trait is implemented, the building blocks will
/// [pop](trait.BuildingBlock.html#tymethod.pop) values in descending
/// order.
pub trait Ordered<V: std::cmp::Ord> {}

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html) to make it thread safe.
///
/// Mark a [`building block`](trait.BuildingBlock.html) as thread safe.
/// When this trait is implemented, the implementer guarantees that the
/// container can be used safely concurrently in between its clones
/// obtained with the method
/// [`Concurrent::clone()`](trait.Concurrent.html#tymethod.clone).
pub trait Concurrent: Send + Sync {
    /// Create a shallow copy of the container pointing to the same
    /// container that can be later used concurrently.
    fn clone(&self) -> Self;
}

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html) to accelerate next lookups
/// for a set of keys.
///
/// This trait provides a
/// [`prefetch()`](trait.Prefetch.html#method.prefetch) method allowing
/// to pay some compute cost upfront to accelerate the next lookup of some
/// keys. This is usefull in a context where a thread runs in the background
/// to reorganize a building block while the building block user is busy
/// doing something else. This feature is also aimed for future
/// implementations of an automatic prefetcher that can predict next
/// accessed keys and prefetch them in the background.
pub trait Prefetch<'a, K: 'a, V: 'a>: BuildingBlock<'a, K, V> {
    /// This the method that will reorganize the building block to
    /// accelerate next lookup of some `keys`. The default implementation
    /// does nothing.
    fn prefetch(&mut self, _keys: Vec<K>) {}

    /// Optimized implementation to take multiple keys out of a building
    /// block. This method returns a vector of all elements matching input
    /// `keys` that were inside a building block. Input keys can be
    /// altered only to remove keys that have been taken out of the
    /// building block.
    /// This method is aimed to accelerate the implementation of
    /// [`prefetch()`](trait.Prefetch.html#method.prefetch) method.
    /// BuildingBlock implementer should make sure to implement this method
    /// if it can be faster than the default implementation.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.iter().filter_map(|k| self.take(k)).collect()
    }
}

/// Policies implementation for `Policy` building block.
pub mod policies;

#[cfg(feature = "stream")]
/// `Stream` and `StreamFactory` implementations
/// to use with `Stream` building block.
pub mod streams;

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
/// We want the [most recently used](../policies/struct.LRU.html) elements
/// to stay in the first layer, and we want to be able to access the
/// container [concurrently](../trait.Concurrent.html).
///
/// Without the builder pattern, such container would be built as follow:
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, BTree, Multilevel, Sequential, Policy};
/// use byoc::policies::{LRU, timestamp::Clock};
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
/// use byoc::policies::{LRU, timestamp::Clock};
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
mod policy;
pub use policy::Policy;
mod profiler;
pub use profiler::{Profiler, ProfilerOutputKind};
mod sequential;
pub use sequential::{Sequential, SequentialCell};
#[cfg(feature = "stream")]
mod stream;
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

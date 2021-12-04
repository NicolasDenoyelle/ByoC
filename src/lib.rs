use std::ops::{Deref, DerefMut};

/// Building block interface for key/value storage.
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
    /// [`Ordered`](policy/trait.Ordered.html) will guarantee the eviction
    /// of elements with the largest value. Usually, such building block
    /// are meant to be wrapped into a [`Policy`](policy/struct.Policy.html)
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

/// Access values inside a [building block](trait.BuildingBlock.html).
///
/// This trait is a companion trait of the
/// [`BuildingBlock`](trait.BuildingBlock.html) trait.
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// Values are wrapped in a RAII guard such that if they are modified,
/// modifications can be forwarded in the building block when for instance,
/// the building block is a key/value container in a file.
/// Values can be accessed by dereferencing the guard into the value inside
/// the building block.
///
/// ## Safety:
///
/// At this time, it does not seam feasible to return a trait object with
/// the same lifetime as the function call. Therefore, any lifetime
/// inference on the returned structure would require it to have the same
/// lifetime as building block instance which would for instance prevent
/// to call this trait method in a loop. As a result, this trait
/// implementation maybe `unsafe`, because the returned guard lifetime
/// may outlive the borrowing lifetime of the container where the inner
/// value originates from.
pub trait Get<K, V, U, W>
where
    U: Deref<Target = V>,
    W: Deref<Target = V> + DerefMut,
{
    /// Get a read-only smart pointer to a value inside the container.
    unsafe fn get(&self, key: &K) -> Option<U>;
    /// Get a smart pointer to a mutable value inside the container.
    unsafe fn get_mut(&mut self, key: &K) -> Option<W>;
}

/// [Building Blocks](trait.BuildingBlock.html)
/// [popping](../trait.BuildingBlock.html#tymethod.pop)
/// values in descending order.
pub trait Ordered<V: std::cmp::Ord> {}

/// Thread safe [building blocks](trait.BuildingBlock.html).
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

/// Storage implementation for key/value pairs.
///
/// As long as a container is not full, it must accept new key/value
/// pairs. Although containers are not required to prevent insertion of
/// duplicate keys, some container implementation may reject a key/value
/// pair if the key is already stored in the container.
pub mod container;

/// Connect two [`BuildingBlock`](../trait.BuildingBlock.html)s.
///
/// Connectors implement the [`BuildingBlock`](trait.BuildingBlock.html)s
/// interface to connect two other building blocks.
/// Connectors typically implement the way data transitions from
/// one stage of the data pipeline to another when calling
/// [`BuildingBlock`](../trait.BuildingBlock.html) methods on a mutable
/// instance.
pub mod connector;

/// [`BuildingBlock`](../trait.BuildingBlock.html) that can be accessed
/// concurrently.
pub mod concurrent;

/// [`BuildingBlock`](../trait.BuildingBlock.html) wrapping another one
/// to profile usage statistics.
pub mod profiler;

/// [`BuildingBlock`](../trait.BuildingBlock.html) implementing a cache
/// policy.
///
/// This container will evict out the nth highest elements when calling
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop) method.
/// [Policy](../policy/struct.Policy.html) is a wrapper implementation of a
/// [building block](../trait.BuildingBlock.html) that
/// [wraps](../policy/trait.ReferenceFactory.html#tymethod.new) /
/// [unwraps](../policy/trait.Reference.html#tymethod.into_inner)
/// values into/from an ordering cell before inserting or removing them
/// in/from the underlying building block.
/// As a result, when values get evicted, they are evicted according to
/// the order defined by the policy of the ordering cell.
///
/// Users must be careful that accessing values wrapped into
/// an order cell might change the order of elements in the container, and
/// therefore, policies should not be used with containers relying on
/// a stable order of its values. Note that containers that rely on a
/// stable order of values should not allow access to their inner values
/// alltogether to avoid this problem.
pub mod policy;

pub mod builder;

/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod private;

/// Public test module available at test time.
/// This module tests the expected behavior of
/// [`BuildinlBlock`](../trait.BuildingBlock.html) and
/// [`Get`](../trait.Get.html) traits with
/// `test_building_block()` and `test_get()`.
#[cfg(test)]
mod tests;

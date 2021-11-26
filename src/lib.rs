use std::ops::{Deref, DerefMut};

/// BuildingBlock trait for Key/Value storage of references.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // BuildingBlock as room for first element and returns None.
/// assert!(c.push(vec![("first", 4)]).pop().is_none());
///
/// // BuildingBlock is full and pops a inserted element.
/// let (key, value) = c.push(vec![("second", 12)]).pop().unwrap();
/// assert!(key == "second");
/// assert!(value == 12);
/// ```
///
/// ## Generics:
///
/// * `K`: Is the key type, used for cache lookups.
/// * `V`: Value to insert in container.
pub trait BuildingBlock<'a, K: 'a, V: 'a> {
    /// Get the number of elements fitting in the container.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a matchig key.
    fn contains(&self, key: &K) -> bool;

    /// Get every values matching key out of the container.
    fn take(&mut self, key: &K) -> Option<(K, V)>;

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the containers,
    /// the returned vector contains all the container values and
    /// the container is left with not value.
    /// Usually implementations will be selecting the
    /// nth max elements in the building block.
    fn pop(&mut self, n: usize) -> Vec<(K, V)>;

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, overflowing values are returned.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a>;
}

pub trait Get<K, V, U, W>
where
    U: Deref<Target = V>,
    W: Deref<Target = V> + DerefMut,
{
    fn get<'a>(&'a self, key: &K) -> Option<U>;
    fn get_mut<'a>(&'a mut self, key: &K) -> Option<W>;
}

/// Key, Value store implementations.
///
/// Containers implement the
/// [`BuildingBlock`](../trait.BuildingBlock.html) interface and
/// provide additional guarantees about the behavior of the implementation.
///
/// * As long as a container is not full, it must accept new key/value
/// pairs. Although containers are not required to prevent insertion of
/// duplicate keys, some container implementation may reject a key/value
/// pair if the key is already stored in the container.
/// * Additionally, containers must guarantee that the victims
/// (key/value pairs) evicted on
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop) are indeed the
/// smallest values.
pub mod container;

/// Connect two other [`BuildingBlock`](../trait.BuildingBlock.html)s.
///
/// Connectors implement the [`BuildingBlock`](../trait.BuildingBlock.html)s
/// interface to connect two other building blocks.
///
/// Connectors typically implement the way data transitions from
/// one stage of the cache to another. For instance, a connector can
/// implement an inclusive cache to facilitate the search for keys
/// in different stages of the cache.
///
/// Unlike containers, connectors do not ensure that insertion are
/// possible even when one of the connected buiding blocks still
/// has room to fit more elements.
pub mod connector;

/// [`BuildingBlock`](../trait.BuildingBlock.html) splitting the key/value
/// pairs traffic toward different other building blocks.
pub mod multiplexer;

/// [`BuildingBlock`](../trait.BuildingBlock.html) that can be accessed
/// concurrently.
pub mod concurrent;

/// [`BuildingBlock`](../trait.BuildingBlock.html) wrapping another one
/// to profile usage statistics.
pub mod profiler;

/// [`BuildingBlock`](../trait.BuildingBlock.html) implementing a cache
/// policy.
///
/// [Container](container/index.html) eviction on
/// [`pop()`](trait.BuildingBlock.html#tymethod.pop) takes the nth highest
/// elements out of the container. [Policy](./policy/struct.Policy.html)
/// is a wrapper implementation of a
/// [building block](trait.BuildingBlock.html) that
/// [wraps](./policy/trait.ReferenceFactory.html#tymethod.wrap)/[unwraps](./policy/trait.Reference.html#tymethod.unwrap)
/// values into/from an ordered cell before inserting or removing them
/// from the underlying building block.
/// As a result, when values get evicted, they are evicted according to
/// the order defined by the policy.
///
/// User of policies must be careful that accessing values wrapped into
/// an order cell might change the order of elements in the container, and
/// therefore, policies should not be used with containers relying on
/// a stable order of its values. Note that containers that rely on a
/// stable order of values should not allow access to their inner values
/// alltogether to avoid this problem.
///
/// ## Examples
///
/// ```
/// ```
pub mod policy;

/// Utils
pub mod utils;

/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod private;

#[cfg(test)]
/// Public test module available only for testing building blocks
/// implementation.
pub mod tests;

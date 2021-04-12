use std::vec::Vec;

/// Container trait.
///
/// ## Generics:
///
/// * `K`: Is the key type, used for cache lookups.
/// * `V`: Value to insert in container.
pub trait Container<K, V> {
    /// Get the number of elements fitting in the container.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a key.
    fn contains(&self, key: &K) -> bool;

    /// Get a value out of the container.
    ///
    /// * `key`: The key associated with the value to take.
    fn take(&mut self, key: &K) -> Option<V>;

    /// Remove a value from the container.
    /// If cache is empty, return None.
    /// Else evict and return a victim key/value pair.
    fn pop(&mut self) -> Option<(K, V)>;

    /// Remove all values from the container.
    fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Insert a value in the container. If an equivalent key already
    /// exists in the container then it is removed before insertion
    /// and returned. Else, if the container was full, a victim is
    /// removed before insertion then returned.
    ///
    /// * `key`: The key associated with the value to insert.
    /// * `value`: The cache value to insert.
    fn push(&mut self, key: K, value: V) -> Option<(K, V)>;

    /// Empty the container and retrieve all elements inside a vector.
    fn flush(&mut self) -> Vec<(K, V)> {
        let mut v = Vec::new();
        loop {
            match self.pop() {
                None => break v,
                Some(x) => v.push(x),
            }
        }
    }
}

/// `get()` method for [containers](trait.Container.html).
///
/// This method looks into the container for a matching key.
/// If key is found, then an item with the same lifetime as the container
/// is returned. This item is intended to hold either a reference or a
/// smart pointer (e.g [cache reference](../reference/trait.Reference.html)
/// or [lock guard](../struct.RWLockGuard.html)) to the matching
/// value inside the container.
/// The accessed container needs to be mutable for several reasons:
/// 1. Because cache references implement interior mutability and update
/// their metadata when they are dereferenced.
/// 2. A returned smart pointer may allow to access a mutable reference
/// to its content.
pub trait Get<'a, K, V>: Container<K, V> {
    type Item: 'a;
    /// Get an item with matching key from cache.
    /// If not found, None is returned.
    /// * `key`: The key value used for searching a value.
    fn get(&'a mut self, key: &K) -> Option<Self::Item>;
}

mod associative;
pub use crate::container::associative::Associative;
mod btree;
pub use crate::container::btree::BTree;
mod filemap;
pub use crate::container::filemap::FileMap;
mod profiler;
pub use crate::container::profiler::Profiler;
mod sequential;
pub use crate::container::sequential::Sequential;
// mod stack;
// pub use crate::container::stack::Stack;
mod vector;
pub use crate::container::vector::Vector;

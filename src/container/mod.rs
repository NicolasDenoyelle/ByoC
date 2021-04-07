use crate::lock::RWLockGuard;
use std::marker::{Send, Sync};
use std::vec::Vec;

/// Container trait.
///
/// ## Generics:
///
/// * `K`: Is the key type, used for cache lookups. Key must be orderable.
/// * `V`: Value to insert in container.
pub trait Container<K, V> {
    /// Get the number of elements fitting in the container.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a key.
    fn contains(&self, key: &K) -> bool;

    /// Get a reference out of the container.
    ///
    /// * `key`: The key associated with the reference to take.    
    fn take(&mut self, key: &K) -> Option<V>;

    /// Remove next reference from the container.
    /// If cache is empty, return None.
    /// Else give ownership to the next reference to evict.
    fn pop(&mut self) -> Option<(K, V)>;

    /// Remove all references from the container.
    fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Insert a reference in the container.
    /// Ownership on reference is taken.
    /// If a reference with the same key is present,
    /// it is returned after replacing it with the new
    /// reference.
    /// If the container is full, then the new reference is inserted
    /// and an evicted reference is returned.
    /// If none of these cases is encountered, None is returned.
    ///
    /// * `key`: The key associated with the reference to insert.
    /// * `reference`: The cache reference to insert.
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

/// Marker trait of a container assessing that if the container hash
/// room for an extra element, then next push will not pop if key
/// is not already in the container. If a container does not implement
/// this trait, the it may pop on trying to push a key that is not already
/// In the container. This is specifically NOT used in
/// [Associative](struct.Associative.html) containers that will pop when
/// inserting in a full set/bucket.
pub trait Packed<K, V: Ord>: Container<K, V> {}

/// `get()` and `get_mut()` methods for sequential
/// [containers](trait.Container.html).
///
/// `get()` and `get_mut()` methods apply to a mutable container reference.
/// Accessing a cache element even read-only, mutates the reference metadata.
/// For instance,
/// [LRFU](../reference/struct.LRFU.html) references require to keep track of
/// the number and [timestamp](../timestamp/trait.Timestamp.html) of accesses.
/// Updates to container references may also mutate the container.
/// For instance [BTree](struct.BTree.html) container maintains a sorted tree of
/// references.
/// When a reference is accessed, references order may change and thus the
/// container is
/// mutated.
pub trait Get<K, V>: Container<K, V> {
    /// Get read-only reference to the content of a cache
    /// [reference](../reference/trait.Reference.html) in the container.
    /// If not found, None is returned.
    /// * `key`: The key value used for searching a reference.
    fn get(&mut self, key: &K) -> Option<&V>;

    /// Get mutable reference to the content of a
    /// cache [reference](../reference/trait.Reference.html) in the container.
    /// If not found, None is returned.
    /// * `key`: The key value used for searching a reference.
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
}

/// Concurrent containers implement `Clone` trait and allow concurrent
/// access in between clones. They also implement `get()`
/// and `get_mut()` methods.
///
/// `get()` and `get_mut()` methods apply to a mutable container reference.
/// Accessing a cache [reference](../reference/trait.Reference.html),
/// even read-only, mutates the reference state via a call to `touch()` on
/// reference.
/// For instance,
/// [LRFU](../reference/struct.LRFU.html) references require to keep track of
/// the number and [timestamp](../timestamp/trait.Timestamp.html) of accesses.
/// Updates to container references may also mutate the container.
/// For instance [BTree](struct.BTree.html) container maintains a sorted tree
/// of references.
/// When a reference is accessed, references order may change and thus the
/// container is mutated.
///
/// Compared to [`Get`](trait.Get.html) trait, this version
/// returns the content of a reference wrapped into a
/// [RWLockGuard](../lock/struct.RWLockGuard.html) that will release a
/// lock once out of scope
pub trait Concurrent<K, V>: Container<K, V> + Clone + Send + Sync {
    /// Get read-only reference to the content of a cache
    /// [reference](../reference/trait.Reference.html) in the container.
    /// If not found, None is returned.
    /// * `key`: The key value used for searching a reference.
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>>;

    /// Get mutable reference to the content of a
    /// cache [reference](../reference/trait.Reference.html) in the container.
    /// If not found, None is returned.
    /// * `key`: The key value used for searching a reference.
    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>>;
}

mod associative;
pub use crate::container::associative::Associative;
mod btree;
pub use crate::container::btree::BTree;
mod filemap;
pub use crate::container::filemap::FileMap;
mod map;
pub use crate::container::map::Map;
mod profiler;
pub use crate::container::profiler::Profiler;
mod sequential;
pub use crate::container::sequential::Sequential;
mod stack;
pub use crate::container::stack::Stack;
mod top_k;
pub use crate::container::top_k::TopK;
mod vector;
pub use crate::container::vector::Vector;

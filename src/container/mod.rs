use crate::lock::RWLockGuard;
use crate::reference::{FromValue, Reference};
use std::marker::{Send, Sync};

/// Container trait.
///
/// ## Generics:
///
/// * `K`: Is the key type, used for cache lookups. Key must be orderable.
/// * `V`: Value type stored in [cache reference](../reference/trait.Reference.html).
/// * `R`: Type of [cache reference](../reference/trait.Reference.html).
pub trait Container<K, V, R>
where
    R: Reference<V>,
{
    /// Get the number of elements fitting in the container.
    fn capacity(&self) -> usize;

    /// Get the number of elements in the container.    
    fn count(&self) -> usize;

    /// Check if container contains a key.
    fn contains(&self, key: &K) -> bool;

    /// Get a reference out of the container.
    ///
    /// * `key`: The key associated with the reference to take.    
    fn take(&mut self, key: &K) -> Option<R>;

    /// Remove next reference from the container.
    /// If cache is empty, return None.
    /// Else give ownership to the next reference to evict.
    fn pop(&mut self) -> Option<(K, R)>;

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
    fn push(&mut self, key: K, reference: R) -> Option<(K, R)>;
}

/// Marker trait of a container assessing that if the container hash
/// room for an extra element, then next push will not pop if key
/// is not already in the container. If a container does not implement
/// this trait, the it may pop on trying to push a key that is not already
/// In the container. This is specifically NOT used in
/// [Associative](struct.Associative.html) containers that will pop when
/// inserting in a full set/bucket.
pub trait Packed<K, V, R>: Container<K, V, R>
where
    R: Reference<V>,
{
}

/// `get()` and `get_mut()` methods for sequential [containers](trait.Container.html).
///
/// `get()` and `get_mut()` methods apply to a mutable container reference.
/// Accessing a cache [reference](../reference/trait.Reference.html),
/// even read-only, mutates the reference state via a call to `touch()` on reference.
/// For instance,
/// [LRFU](../reference/struct.LRFU.html) references require to keep track of
/// the number and [timestamp](../timestamp/trait.Timestamp.html) of accesses.
/// Updates to container references may also mutate the container.
/// For instance [BTree](struct.BTree.html) container maintains a sorted tree of references.
/// When a reference is accessed, references order may change and thus the container is
/// mutated.
pub trait Sequential<K, V, R>: Container<K, V, R>
where
    R: Reference<V>,
{
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

/// Implementation of direct insertion of values inside container.
///
/// Implementer of container can add this capability to containers for free:
/// ```ignore
/// use crate::reference::{Reference, FromValue};
/// use crate::container::{Container, Insert};
/// impl<'a,K,V,R: Reference<V> + FromValue<V>> Insert<'a,K,V,R> for MyContainer<K,V,R> {}
/// ```
pub trait Insert<K, V, R>: Container<K, V, R>
where
    R: Reference<V> + FromValue<V>,
{
    fn insert(&mut self, key: K, value: V) -> Option<(K, R)> {
        let reference = R::from_value(value);
        self.push(key, reference)
    }
}

/// From ref mut [Container](trait.Container.html) into iterator of non mutable values.
pub trait Iter<'a, K, V, R>: Container<K, V, R>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator: Iterator<Item = (&'a K, &'a V)>;
    fn iter(&'a mut self) -> Self::Iterator;
}

/// From ref mut [Container](trait.Container.html) into iterator of mutable values.
pub trait IterMut<'a, K, V, R>: Container<K, V, R>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
{
    type Iterator: Iterator<Item = (&'a K, &'a mut V)>;
    fn iter_mut(&'a mut self) -> Self::Iterator;
}

/// `get()` and `get_mut()` methods for thread safe containers.
///
/// `get()` and `get_mut()` methods apply to a mutable container reference.
/// Accessing a cache [reference](../reference/trait.Reference.html),
/// even read-only, mutates the reference state via a call to `touch()` on reference.
/// For instance,
/// [LRFU](../reference/struct.LRFU.html) references require to keep track of
/// the number and [timestamp](../timestamp/trait.Timestamp.html) of accesses.
/// Updates to container references may also mutate the container.
/// For instance [BTree](struct.BTree.html) container maintains a sorted tree of references.
/// When a reference is accessed, references order may change and thus the container is
/// mutated.
///
/// This version returns the content of a reference wrapped into a
/// [RWLockGuard](../lock/struct.RWLockGuard.html) that will release a lock once
/// out of scope
pub trait Concurrent<K, V, R>:
    Container<K, V, R> + Clone + Send + Sync
where
    R: Reference<V>,
{
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

pub mod concurrent;
pub mod sequential;

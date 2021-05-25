use crate::container::Container;

/// Marker trait for a [`containers`](../container/trait.Container.html).
/// When this trait is implemented, the implementer guarantees that
/// if the container has room for an extra element, then next push will
/// not pop. If a container does not implement this trait, the it may pop
/// when trying to push a new key/value pair even if the container is not
/// full. This is specifically NOT used in
/// [Associative](../container/struct.Associative.html) containers that
/// will pop when inserting in a full set/bucket.
pub trait Packed<'a, K: 'a, V: 'a>: Container<'a, K, V> {}

/// Marker trait for a [`containers`](../container/trait.Container.html).
/// When this trait is implemented, the implementer guarantees that the
/// container can be used safely concurrently in between its clones.
/// Clones of this container are shallow copies referring to the same
/// storage.  
/// This marker traits combines the traits: `Send`, `Sync` and `Clone`.
/// This traits
pub trait Concurrent<'a, K: 'a, V: 'a>:
    Container<'a, K, V> + Clone + Send + Sync
{
}

use crate::container::Container;

/// Marker trait of a container assessing that if the container hash
/// room for an extra element, then next push will not pop if key
/// is not already in the container. If a container does not implement
/// this trait, the it may pop on trying to push a key that is not already
/// In the container. This is specifically NOT used in
/// [Associative](struct.Associative.html) containers that will pop when
/// inserting in a full set/bucket.
pub trait Packed<'a, K: 'a, V: 'a>: Container<'a, K, V> {}

/// Concurrent containers implement `Clone` trait and allow concurrent
/// access in between clones.
pub trait Concurrent<'a, K: 'a, V: 'a>:
    Container<'a, K, V> + Clone + Send + Sync
{
}

/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod utils;
#[cfg(feature = "stream")]
pub use utils::io::{IOError, IOResult, IOStruct, IOStructMut};

/// Library custom read/write lock.
mod lock;
pub use lock::RWLockGuard;

/// Fixed point in time.
///
/// ## Details
///
/// [`Timestamp`](trait.Timestamp.html) is a trait that represent a
/// point in time, such that consecutive timestamps are getting greater as the time flies.
/// Timestamp module contains [`Timestamp`](trait.Timestamp.html) trait
/// definition and implementations as [`Counter`](struct.Counter.html) and
/// [Clock](struct.Clock.html). [timestamp](index.html) module
/// is used to implement [LRFU](../reference/struct.LRFU.html)
/// cache [references](../reference/trait.Reference.html).
pub mod timestamp;

/// Cache references.
///
/// A cache reference is a smart pointer trait for struct implementing
/// `Ord`, `Deref` and `DerefMut` traits. A cache reference may implement
/// interior mutability such that derefencing it may update the reference
/// internal state and order with other references. This typically used,
/// for instance, to find most the recently used value as implemented by
/// [LRU](reference/struct.LRU.html) reference.
///
/// ## Examples
///
/// Least Recently Used references will be updated when they are used.
/// ```
/// use std::ops::Deref;
/// use cache::reference::{Reference, LRU};
///
/// let mut r0 = LRU::new("first reference");
/// let r1 = LRU::new("second reference");
///
/// // r0 is the least recently created reference.
/// assert!( r0 > r1 );
///
/// // Access r0 to make it the most recently used.
/// assert!( r0.deref() == &"first reference" );
///
/// // Now r1 is the least recently used reference.
/// assert!( r0 < r1 );
/// ```
pub mod reference;

/// Key/Value storage of references.
///
/// ## Details
///
/// A container is a key/value storage with set maximum capacity.
/// When maximum capacity is reached, new insertion cause the container
/// to evict the element with the highest value in the container.
/// Each container implementation implements a specific way to perform
/// insertions and lookups, and target a specific storage tier.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Vector};
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push(vec![("first", 4)]).pop().is_none());
///
/// // Container is full and pops a inserted element.
/// let (key, value) = c.push(vec![("second", 12)]).pop().unwrap();
/// assert!(key == "second");
/// assert!(value == 12);
/// ```
pub mod container;

/// Marker traits for containers.
/// Marker traits are traits that combine several traits without
/// implementing new methods.
pub mod marker;

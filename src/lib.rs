/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod utils;

/// Library custom read/write lock.
pub mod lock;

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

/// Cache references (value + eviction policy).
///
/// ## Details
///
/// A cache reference is a smart pointer implementing a cache eviction policy.
/// New cache policies can easily be implemented by implementing
/// [Reference](trait.Reference.html) trait.
///
/// Access of element inside a cache reference is implemented through
/// `Deref` and `DerefMut` traits.
/// The owner of a cache reference can destroy it and obtain ownership on its
/// inner data by using `unwrap()` method.
///
/// Whenever a reference is accessed via a container, the latter will update the
/// reference via a call to `touch()` method. This call will update the reference
/// state and its likelihood to be the next victim.
///
/// Cache eviction policy is implemented with the `Ord` trait on
/// [`Reference`](trait.Reference.html).
/// Whenever an election of cache victim has to be done, the maximum value for
/// cache references according to `Ord` implementation is elected as the victim.
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
/// assert!( r0.deref() == &"first reference" );
/// r0.touch();
///
/// // Now r1 is the least recently used reference.
/// assert!( r0 < r1 );
/// ```
pub mod reference;

/// Key/Value storage of references.
///
/// ## Details
///
/// A container is a key/value storage for
/// cache [references](../reference/trait.Reference.html).
/// It implements a specific way to perform insertion and lookup of cache
/// references. Depending on the references access patterns (amount of references,
/// random or predictible pattern), one storage may be more performant than another.
///
/// Container module provides trait definition and few implementations of containers.
/// To mention a few, implementations include [`Associative`](struct.Associative.html)
/// container, [`ordered`](struct.BTree.html) (keys and references) container.
/// Caches performance can further be improved by providing new container implementations.
///
/// Cache containers have the particularity of having a set size.
/// Whenever a new [reference](../reference/trait.Reference.html) inserted in a container,
/// an existing reference may be popped if the container is full or contain a duplicate key.
///
/// ## Examples
///
/// ```
/// use cache::container::Container;
/// use cache::container::sequential::Vector;
/// use cache::reference::{Reference, Default};
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", Default::new(4)).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", Default::new(12)).unwrap();
///
/// // The victim is the second reference because it has a greater value.
/// assert!(key == "second");
/// assert!(*value == 12);
/// ```
pub mod container;

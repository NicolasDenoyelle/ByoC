/// BuildingBlock trait for Key/Value storage of references.
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
/// use cache::BuildingBlock;
/// use cache::building_block::container::Vector;
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
    ///
    /// * `key`: The key associated with the values to take.
    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b>;

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the containers,
    /// the returned vector contains all the container values and
    /// the container is left with not value.
    fn pop(&mut self, n: usize) -> Vec<(K, V)>;

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, overflowing values are returned.
    ///
    /// * `key`: The key associated with the value to insert.
    /// * `values`: The cache values to insert.
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a>;
}

pub trait Get<'a, K: 'a, V: 'a>: BuildingBlock<'a, K, V> {
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b>;

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b>;
}

/// Implementations of building blocks.
pub mod building_block;

/// Cache references.
///
/// A cache reference is a smart pointer trait for struct implementing
/// `Ord`, `Deref` and `DerefMut` traits. A cache reference may implement
/// interior mutability such that derefencing it may update the reference
/// internal state and order with other references. This typically used,
/// for instance, to find most the recently used value as implemented by
/// [LRU](./struct.LRU.html) reference.
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

/// Utils
pub mod utils;

/// Library boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod private;

// /// Public test module available only for testing purposes.
#[cfg(test)]
pub mod tests;

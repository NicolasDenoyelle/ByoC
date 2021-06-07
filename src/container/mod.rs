/// Container trait.
///
/// ## Generics:
///
/// * `K`: Is the key type, used for cache lookups.
/// * `V`: Value to insert in container.
pub trait Container<'a, K: 'a, V: 'a> {
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

    /// Remove a value from the container.
    /// If cache is empty, return None.
    /// Else evict and return a victim key/value pair.
    fn pop(&mut self) -> Option<(K, V)>;

    /// Remove all values from the container.
    fn clear(&mut self) {
        #[allow(unused_must_use)]
        {
            self.flush();
        }
    }

    /// Insert a key/value pair in the container. If the container was
    /// full, a victim is removed before insertion then returned.
    ///
    /// * `key`: The key associated with the value to insert.
    /// * `value`: The cache value to insert.
    fn push(&mut self, key: K, value: V) -> Option<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    /// The container becomes empty and available at the end of the call.
    /// This functions yields an iterator because the amount of items to
    /// iterate over might exceed the size of computer memory.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a>;
}

pub trait Get<'a, K: 'a, V: 'a>: Container<'a, K, V> {
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b>;

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b>;
}

//------------------------------------------------------------------------//
/// `Buffered` trait.
/// Containers implementing this trait can optimize
/// [`push()`](trait.Container.html#method.push) method when inserting
/// multiple elements at once.  
/// For instance the
/// [`FileMap`](./sequential/struct.FileMap.html) container walks
/// the entire file when pushing a (key, value) pair.
/// This trait implementation will also walk the entire file once but
/// will perform all possible insertions meanwhile.
pub trait Buffered<'a, K: 'a, V: 'a>: Container<'a, K, V> {
    /// Insert a buffer of keys and value in the container.
    /// Returns a vector of victims if the container overflows.
    /// Any container can implement this trait for free.
    /// The default implementation will use
    /// [`push()`](trait.Container.html#method.push) method on each
    /// element in input.
    fn push_buffer(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut out = Vec::new();
        for (k, v) in elements {
            match self.push(k, v) {
                None => (),
                Some(x) => out.push(x),
            }
        }
        out
    }
}

//------------------------------------------------------------------------//
// Containers implementations.
//------------------------------------------------------------------------//

mod associative;
pub use crate::container::associative::Associative;
mod btree;
pub use crate::container::btree::BTree;
mod profiler;
pub use crate::container::profiler::Profiler;
mod sequential;
pub use crate::container::sequential::Sequential;
mod stack;
pub use crate::container::stack::Stack;
mod vector;
pub use crate::container::vector::Vector;
#[cfg(feature = "filemap")]
mod filemap;
#[cfg(feature = "filemap")]
pub use crate::container::filemap::FileMap;

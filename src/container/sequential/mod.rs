mod vector;
pub use crate::container::sequential::vector::Vector;
mod map;
pub use crate::container::sequential::map::Map;
mod btree;
pub use crate::container::sequential::btree::{BTree, BTreeIterator};
mod stack;
pub use crate::container::sequential::stack::Stack;
mod top_k;
pub use crate::container::sequential::top_k::TopK;
/// Reading and writing objects.
///
/// ## Details
///
/// The IO module provides the abstractions for reading and writing objects
/// in mediums other than main memory.
/// ```
mod io;

#[cfg(test)]
mod tests;

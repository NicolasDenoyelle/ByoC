use std::collections::LinkedList;

/// A list of building blocks in a building block.
///
/// `Batch` stores multiple building blocks of the same kind in a list.
/// The goal of this building block is to cut the cost of accessing a
/// building block into small pieces. Specifically, when the underlying
/// building block is a [`Compressor`](struct.Compressor.html), only small
/// part of a building block are decompressed into memory, thus reducing
/// the memory footprint.
pub struct Batch<C> {
    pub(super) bb: LinkedList<C>,
}

impl<C> Batch<C> {
    /// Build an empty batch of building blocks.
    pub fn new() -> Self {
        Batch {
            bb: LinkedList::new(),
        }
    }

    /// Append a building block to the batch.
    pub fn append(&mut self, c: C) {
        self.bb.push_back(c);
    }
}

impl<T, const N: usize> From<[T; N]> for Batch<T> {
    fn from(arr: [T; N]) -> Self {
        Batch {
            bb: LinkedList::from(arr),
        }
    }
}

impl<C> Default for Batch<C> {
    fn default() -> Self {
        Self::new()
    }
}

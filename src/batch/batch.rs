use std::collections::LinkedList;

/// Chunk a big `BuildingBlock` into small batches.
///
/// This container is implemented as an array of building blocks.
/// Each [`BuildingBlock`](trait.BuildingBlock.html) of this
/// [`Batch`] container is named a batch.
/// The goal of this container is to cut the cost of accessing a
/// building block into small pieces (or batches). Specifically, when the
/// underlying building block is a [`Compressed`](struct.Compressed.html),
/// the whole container is unpacked in the main memory.
/// Therefore, chunking such a containers into small batches will help
/// reducing the memory footprint and enable managing data that would otherwise
/// be unmanageable due to its size.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// [`Get`](trait.Get.html) and [`Concurrent`](trait.Concurrent.html)
/// traits are inherited from the type of container used to build this
/// batch container. [`Get`](trait.Get.html) trait works similarly as
/// [`take()`](trait.BuildingBlock.html#tymethod.take) method. It iterates
/// batches from front to back and stops at the first match.
///
/// ## Examples
///
/// ```
/// use byoc::{Batch, BuildingBlock, Array};
///
/// let mut batch = Batch::new().append(Array::new(1))
///                             .append(Array::new(1));
///
/// assert_eq!(batch.push(vec![(0,"first"), (1,"second")]).len(), 0);
/// assert_eq!(batch.take(&1).unwrap().1, "second");
/// assert_eq!(batch.take(&0).unwrap().1, "first");
/// ```
///
/// [`Batch`] can also be built from a
/// [configuration](config/struct.BatchConfig.html).
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
    pub fn append(mut self, c: C) -> Self {
        self.bb.push_back(c);
        self
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

impl<'a, K, V, C> From<Batch<C>> for crate::DynBuildingBlock<'a, K, V>
where
    K: 'a,
    V: 'a + Ord,
    C: 'a + crate::BuildingBlock<K, V>,
{
    fn from(batch: Batch<C>) -> Self {
        crate::DynBuildingBlock::new(batch, false)
    }
}

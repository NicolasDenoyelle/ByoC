use super::DynConcurrent;
use crate::BuildingBlock;
use std::marker::PhantomData;

// Intermediate struct to make the associated type of BuildingBlock object safe.
#[repr(transparent)]
struct DynIterBuildingBlock<'a, C> {
    container: C,
    unused: PhantomData<&'a bool>,
}

impl<'a, K, V, C, F: 'a> BuildingBlock<K, V>
    for DynIterBuildingBlock<'a, C>
where
    F: Iterator<Item = (K, V)>,
    C: BuildingBlock<K, V, FlushIterator = F>,
{
    type FlushIterator = Box<dyn Iterator<Item = (K, V)> + 'a>;
    fn capacity(&self) -> usize {
        self.container.capacity()
    }
    fn size(&self) -> usize {
        self.container.size()
    }
    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.container.take(key)
    }
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.container.pop(n)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        self.container.push(values)
    }
    fn flush(&mut self) -> Self::FlushIterator {
        Box::new(self.container.flush())
    }
}

/// A pseudo-object-safe [`BuildingBlock`] wrapper.
///
/// This object encapsulate a boxed container that yields boxed associated types
/// when they are used.
///
/// It may be evolved into a pseudo-object-safe [`BuildingBlock`] that carries
/// the [`Concurrent`](../trait.Concurrent.html) trait using its
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent)
/// method.
///
/// It may only be built with the [`std::convert::From`] trait implementers who
/// know whether it is safe to convert this object into a [`DynConcurrent`]
/// object.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, DynBuildingBlock, Array, Sequential};
///
/// // Creation of a boxed array with boxed associated types.
/// let dyn_array = DynBuildingBlock::from(Array::<(u64,u64)>::new(10));
///
/// // Array is not a concurrent container and thus, its object safe variant
/// // is not concurrent either.
/// assert!(dyn_array.into_concurrent().is_err());
///
/// // Creation of a boxed concurrent array.
/// let dyn_sequential = DynBuildingBlock::from(Sequential::new(Array::<(u64,u64)>::new(10)));
/// assert!(dyn_sequential.into_concurrent().is_ok());
/// ```
#[allow(clippy::type_complexity)]
pub struct DynBuildingBlock<'a, K, V> {
    building_block: Box<
        dyn BuildingBlock<
                K,
                V,
                FlushIterator = Box<dyn Iterator<Item = (K, V)> + 'a>,
            > + 'a,
    >,
    has_concurrent_trait: bool,
}

impl<'a, K, V> DynBuildingBlock<'a, K, V> {
    /// Create a [`DynBuildingBlock`] from a container, specifying whether it
    /// should be possible to clone the box pointer to use it concurrently.
    pub(crate) fn new<
        F: Iterator<Item = (K, V)> + 'a,
        C: 'a + BuildingBlock<K, V, FlushIterator = F>,
    >(
        container: C,
        has_concurrent_trait: bool,
    ) -> Self {
        Self {
            building_block: Box::new(DynIterBuildingBlock {
                container,
                unused: PhantomData,
            }),
            has_concurrent_trait,
        }
    }

    // /// Returns whether the object can be succefully made into a
    // /// [`DynConcurrent`] object.
    // pub(crate) fn has_concurrent_trait(&self) -> bool {
    //     self.has_concurrent_trait
    // }

    /// Make this [`BuildingBlock`](../trait.BuildingBlock.html) into an
    /// [`Concurrent`](../traits/trait.Concurrent.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    ///
    /// If the object does not support the trait, an error is returned.
    pub fn into_concurrent(self) -> Result<DynConcurrent<Self>, Self> {
        if self.has_concurrent_trait {
            Ok(DynConcurrent::new(self))
        } else {
            Err(self)
        }
    }
}

impl<'a, K: 'a, V: 'a> BuildingBlock<K, V> for DynBuildingBlock<'a, K, V> {
    fn capacity(&self) -> usize {
        self.building_block.capacity()
    }
    fn size(&self) -> usize {
        self.building_block.size()
    }
    fn contains(&self, key: &K) -> bool {
        self.building_block.contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.building_block.take(key)
    }
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        self.building_block.take_multiple(keys)
    }
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.building_block.pop(n)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        self.building_block.push(values)
    }

    type FlushIterator = Box<dyn Iterator<Item = (K, V)> + 'a>;
    fn flush(&mut self) -> Self::FlushIterator {
        self.building_block.flush()
    }
}

use crate::internal::lock::RWLock;
use crate::internal::SharedPtr;

/// `Concurrent` `BuildingBlock` wrapper with a lock.
///
/// This wrapper can be used to makes a container thread safe by
/// sequentializing its access.
///
/// This building block can also be built with a
/// [builder](builder/trait.Build.html#method.into_sequential) pattern or from a
/// [configuration](config/configs/struct.SequentialConfig.html).
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// This is a simple wrapper with a read/write lock. Shared methods call
/// lock the lock for reading while exclusive accesses lock the lock for writing
/// for the length of the method call.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// Same as for [`BuildingBlock`](trait.BuildingBlock.html) Implementation.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Concurrent};
/// use byoc::{Array, Sequential};
///
/// // Build a concurrent Array cache.
/// let mut c1 = Sequential::new(Array::new(1));
/// let mut c2 = Concurrent::clone(&c1);
///
/// assert!(c1.push(vec![(0u16, 4)]).pop().is_none());
/// let (key, value) = c2.push(vec![(1u16, 12)]).pop().unwrap();
/// assert_eq!(key, 0u16);
/// assert_eq!(value, 4);
///```
pub struct Sequential<C> {
    pub(super) container: SharedPtr<C>,
    pub(super) lock: RWLock,
}

impl<C: Clone> Clone for Sequential<C> {
    fn clone(&self) -> Self {
        Sequential {
            container: SharedPtr::from(self.container.as_ref().clone()),
            lock: RWLock::new(),
        }
    }
}

impl<C> Sequential<C> {
    /// Construct a new concurrent container wrapping an existing
    /// `container`.
    pub fn new(container: C) -> Self {
        Sequential {
            container: SharedPtr::from(container),
            lock: RWLock::new(),
        }
    }
}

impl<'a, K, V, C> From<Sequential<C>> for crate::DynBuildingBlock<'a, K, V>
where
    K: 'a,
    V: 'a,
    C: 'a + crate::BuildingBlock<K, V>,
{
    fn from(sequential: Sequential<C>) -> Self {
        crate::DynBuildingBlock::new(sequential, true)
    }
}

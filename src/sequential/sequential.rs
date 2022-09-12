use crate::internal::lock::RWLock;
use crate::internal::SharedPtr;

/// [`Concurrent`](trait.Concurrent.html)
/// [`BuildingBlock`](trait.BuildingBlock.html) wrapper with a lock.
///
/// This wrapper can be used to makes a container thread safe by
/// sequentializing its access.
///
/// This building block can also be built with a
/// [builder](builder/trait.Build.html#method.into_sequential) pattern or from a
/// [configuration](config/configs/struct.SequentialConfig.html).
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Concurrent};
/// use byoc::{Array, Sequential};
///
/// // Build a concurrent Array cache.
/// let element_size = Array::<(u16,u32)>::element_size();
/// let mut c1 = Sequential::new(Array::new(element_size));
/// let mut c2 = Concurrent::clone(&c1);
///
/// assert!(c1.push(vec![(0u16, 4)]).pop().is_none());
/// let (key, value) = c2.push(vec![(1u16, 12)]).pop().unwrap();
/// assert_eq!(key, 1u16);
/// assert_eq!(value, 12);
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

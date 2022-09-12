use crate::config::ConfigError;
use crate::internal::lock::RWLock;
use crate::policy::Ordered;
use crate::{BuildingBlock, Concurrent};
use std::cmp::Ord;

/// A Boxed [`BuildingBlock`](../trait.BuildingBlock.html) trait.
///
/// This object is built from a [config](index.html)
/// [`Builder`](struct.Builder.html). This object is a wrapper around a boxed
/// [`BuildingBlock`](../trait.BuildingBlock.html) trait that implements
/// the latter.
/// Additionally, this object can be made into an object that also implements
/// the [`Ordered`](../policy/trait.Ordered.html) or the
/// [`Concurrent`](../trait.Concurrent.html) trait if the container that
/// is described by the initial configuration implements the latter.
///
/// ## Examples
///
/// Making a valid configuration into a
/// [`BuildingBlock`](../trait.BuildingBlock.html) that carries the
/// [`Ordered`](../policy/trait.Ordered.html) trait or/and the
/// [`Concurrent`](../trait.Concurrent.html) trait.
/// ```
/// use byoc::config::{Builder, DynBuildingBlock, DynConcurrent, DynOrdered};
///
/// // Write a `Concurrent` and `Ordered` container configuration
/// // and make a `Builder` object from it.
/// let config = "
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ";
/// // The configuration is valid so we can `unwrap()`.
/// let builder = Builder::from_string(config).unwrap();
///
/// // This configuration can be made into an `Ordered` `BuildingBlock`.
/// let container: DynOrdered<DynBuildingBlock<u64,u64>> =
///                builder.clone().build().ordered().unwrap();
///
/// // This configuration can be made into an `Concurrent` `BuildingBlock`.
/// let container: DynConcurrent<DynBuildingBlock<u64,u64>> =
///                builder.clone().build().concurrent().unwrap();
///
/// // This configuration can be made both `Concurrent` and `Ordered`.
/// let container: DynConcurrent<DynOrdered<DynBuildingBlock<u64,u64>>> =
///                builder.clone()
///                       .build()
///                       .ordered()
///                       .unwrap()
///                       .concurrent()
///                       .unwrap();
/// ```
///
/// Not all the configurations can support these traits.
/// ```
/// use byoc::config::{Builder,
///                    ConfigError,
///                    DynBuildingBlock,
///                    DynConcurrent,
///                    DynOrdered};
///
/// // Write a `Concurrent` and `Ordered` container configuration
/// // and make a `Builder` object from it.
/// let config = "
/// id='BTreeConfig'
/// capacity=10
/// ";
/// // The configuration is valid so we can `unwrap()`.
/// let builder = Builder::from_string(config).unwrap();
///
/// // This configuration cannot be made into an `Ordered` `BuildingBlock`.
/// let result: Result<DynOrdered<DynBuildingBlock<u64,u64>>, ConfigError> =
///          builder.clone().build().ordered();
/// assert!(result.is_err());
///
/// // This configuration cannot be made into an `Concurrent` `BuildingBlock`.
/// let result: Result<DynConcurrent<DynBuildingBlock<u64,u64>>, ConfigError> =
///          builder.clone().build().concurrent();
/// assert!(result.is_err());
/// ```
pub struct DynBuildingBlock<'a, K, V> {
    building_block: Box<dyn BuildingBlock<'a, K, V> + 'a>,
    has_concurrent_trait: bool,
    has_ordered_trait: bool,
}

impl<'a, K, V> DynBuildingBlock<'a, K, V> {
    pub(super) fn new(
        building_block: Box<dyn BuildingBlock<'a, K, V> + 'a>,
        has_concurrent_trait: bool,
        has_ordered_trait: bool,
    ) -> Self {
        Self {
            building_block,
            has_concurrent_trait,
            has_ordered_trait,
        }
    }

    /// Make this [`BuildingBlock`](../trait.BuildingBlock.html) into an
    /// [`Ordered`](../policy/trait.Ordered.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    /// If the initial configuration that was used to make this
    /// [`DynBuildingBlock`] did not support the
    /// [`Ordered`](../policy/trait.Ordered.html) trait, a
    /// [`ConfigError::UnsupportedTraitError`] is returned, else,
    /// a [`BuildingBlock`](../trait.BuildingBlock.html) that carries
    /// the [`Ordered`](../policy/trait.Ordered.html) trait is returned.
    pub fn ordered(self) -> Result<DynOrdered<Self>, ConfigError> {
        let has_concurrent_trait = self.has_concurrent_trait;
        if self.has_ordered_trait {
            Ok(DynOrdered::new(self, has_concurrent_trait))
        } else {
            Err(ConfigError::UnsupportedTraitError(String::from(
                "This container configuration does not support the Ordered trait."),
            ))
        }
    }

    /// Make this [`BuildingBlock`](../trait.BuildingBlock.html) into an
    /// [`Concurrent`](../policy/trait.Concurrent.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    /// If the initial configuration that was used to make this
    /// [`DynBuildingBlock`] did not support the
    /// [`Concurrent`](../trait.Concurrent.html) trait, a
    /// [`ConfigError::UnsupportedTraitError`] is returned, else,
    /// a [`BuildingBlock`](../trait.BuildingBlock.html) that carries
    /// the [`Concurrent`](../trait.Concurrent.html) trait is returned.
    pub fn concurrent(self) -> Result<DynConcurrent<Self>, ConfigError> {
        let has_ordered_trait = self.has_ordered_trait;
        if self.has_concurrent_trait {
            Ok(DynConcurrent::new(self, has_ordered_trait))
        } else {
            Err(ConfigError::UnsupportedTraitError(String::from(
                "This container configuration does not support the Concurrent trait."),
            ))
        }
    }
}

impl<'a, K, V> BuildingBlock<'a, K, V> for DynBuildingBlock<'a, K, V>
where
    K: 'a,
    V: 'a,
{
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
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.building_block.flush()
    }
}

/// A wrapper around a [`DynBuildingBlock`] that provides the
/// [`Ordered`](../policy/trait.Ordered.html) trait.
///
/// This object is obtained from building a [config](index.html)
/// [`Builder`](struct.Builder.html) object into a [`DynBuildingBlock`]
/// and further calling the
/// [`ordered()`](struct.DynBuildingBlock.html#method.ordered) method
/// of the [`DynBuildingBlock`] object.
///
/// This object can be made into an object that also implements
/// the[`Concurrent`](../trait.Concurrent.html) trait if the container that
/// is described by the initial configuration implements the latter.
///
/// ## Examples
///
/// ```
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// // Build a container from a configuration.
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string("
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ").unwrap().build();
///
/// // Array containers are `Ordered` so it is ok to call the `ordered()`
/// // method.
/// let container = container.ordered().unwrap();
///
/// // Sequential container is a `Concurrent` container so it is ok to call
/// // the `concurrent()` method.
/// let container = container.concurrent().unwrap();
/// ```
pub struct DynOrdered<C> {
    pub(super) building_block: C,
    has_concurrent_trait: bool,
}

impl<C> DynOrdered<C> {
    pub(crate) fn new(
        building_block: C,
        has_concurrent_trait: bool,
    ) -> Self {
        Self {
            building_block,
            has_concurrent_trait,
        }
    }

    /// Make this [`Ordered`](../policy/trait.Ordered.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html) into a
    /// [`Concurrent`](../policy/trait.Concurrent.html) and
    /// [`Ordered`](../policy/trait.Ordered.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    /// If the initial configuration that was used to make this
    /// [`DynBuildingBlock`] did not support the
    /// [`Concurrent`](../trait.Concurrent.html) trait, a
    /// [`ConfigError::UnsupportedTraitError`] is returned, else,
    /// a [`BuildingBlock`](../trait.BuildingBlock.html) that carries
    /// the [`Concurrent`](../trait.Concurrent.html) trait and the
    /// [`Ordered`](../policy/trait.Ordered.html) trait is returned.
    pub fn concurrent(self) -> Result<DynConcurrent<Self>, ConfigError> {
        if self.has_concurrent_trait {
            Ok(DynConcurrent::new(self, true))
        } else {
            Err(ConfigError::UnsupportedTraitError(String::from(
                "This container configuration does not support the Concurrent trait."),
            ))
        }
    }
}

impl<'a, K, R, C> BuildingBlock<'a, K, R> for DynOrdered<C>
where
    K: 'a,
    R: 'a,
    C: BuildingBlock<'a, K, R>,
{
    fn capacity(&self) -> usize {
        self.building_block.capacity()
    }
    fn size(&self) -> usize {
        self.building_block.size()
    }
    fn contains(&self, key: &K) -> bool {
        self.building_block.contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, R)> {
        self.building_block.take(key)
    }
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, R)> {
        self.building_block.take_multiple(keys)
    }
    fn pop(&mut self, n: usize) -> Vec<(K, R)> {
        self.building_block.pop(n)
    }
    fn push(&mut self, values: Vec<(K, R)>) -> Vec<(K, R)> {
        self.building_block.push(values)
    }
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, R)> + 'a> {
        self.building_block.flush()
    }
}

impl<R: Ord, C> Ordered<R> for DynOrdered<C> {}

impl<C: Concurrent> Concurrent for DynOrdered<C> {
    fn clone(&self) -> Self {
        Self {
            building_block: self.building_block.clone(),
            has_concurrent_trait: self.has_concurrent_trait,
        }
    }
}

/// A wrapper around a [`DynBuildingBlock`] that provides the
/// [`Concurrent`](../trait.Concurrent.html) trait.
///
/// This object is obtained from building a [config](index.html)
/// [`Builder`](struct.Builder.html) object into a [`DynBuildingBlock`]
/// and further calling the
/// [`concurrent()`](struct.DynBuildingBlock.html#method.concurrent) method
/// of the [`DynBuildingBlock`] object.
///
/// This object can be made into an object that also implements
/// the[`Ordered`](../policy/trait.Ordered.html) trait if the container that
/// is described by the initial configuration implements the latter.
///
/// Under the hood, this structure wraps a reference counter on a pointer to
/// a [`DynBuildingBlock`] object. When the
/// [`Concurrent::clone()`](../trait.Concurrent.html#tymethod.clone) method
/// is called, the reference count is incremented and the pointer copied.
/// When the last clone goes out of scope, the pointer is freed.
/// This can only work safely if the pointer points to a building block that
/// can effectively be safely used concurrently without needing to check
/// borrowing rules. This is supposedly checked by the configuration
/// [`Builder`](struct.Builder.html).
///
/// ## Examples
///
/// ```
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// // Build a container from a configuration.
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string("
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ").unwrap().build();
///
/// // Sequential containers are `Concurrent` so it is ok to call the
/// // `concurrent()` method.
/// let container = container.concurrent().unwrap();
///
/// // Array container is an `Ordered` container and therefore, so it the
/// // Sequential container. It is ok to call the `ordered()` method.
/// let container = container.ordered().unwrap();
/// ```
pub struct DynConcurrent<C> {
    building_block: *mut C,
    has_ordered_trait: bool,
    rc: RWLock,
}

impl<C> DynConcurrent<C> {
    pub(crate) fn new(bb: C, has_ordered_trait: bool) -> Self {
        let rc = RWLock::new();
        rc.lock().unwrap();
        let bb = Box::into_raw(Box::new(bb));
        Self {
            building_block: bb,
            has_ordered_trait,
            rc,
        }
    }

    /// Make this [`BuildingBlock`](../trait.BuildingBlock.html) into an
    /// [`Ordered`](../policy/trait.Ordered.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    /// If the initial configuration that was used to make this
    /// [`DynConcurrent`]`<`[`DynBuildingBlock`]`>` did not support the
    /// [`Ordered`](../policy/trait.Ordered.html) trait, a
    /// [`ConfigError::UnsupportedTraitError`] is returned, else,
    /// a [`BuildingBlock`](../trait.BuildingBlock.html) that carries
    /// the [`Ordered`](../policy/trait.Ordered.html) trait and the
    /// [`Concurrent`](../trait.Concurrent.html) trait is returned.
    pub fn ordered(self) -> Result<DynOrdered<Self>, ConfigError> {
        if self.has_ordered_trait {
            Ok(DynOrdered::new(self, true))
        } else {
            Err(ConfigError::UnsupportedTraitError(String::from(
                "This container configuration does not support the Ordered trait."),
            ))
        }
    }
}

impl<C> Drop for DynConcurrent<C> {
    fn drop(&mut self) {
        if self.rc.try_lock_mut().is_ok() {
            unsafe { drop(Box::from_raw(self.building_block)) };
            self.rc.unlock();
        }
    }
}

impl<R: Ord, C: Ordered<R>> Ordered<R> for DynConcurrent<C> {}

unsafe impl<C> Send for DynConcurrent<C> {}

unsafe impl<C> Sync for DynConcurrent<C> {}

impl<C> Concurrent for DynConcurrent<C> {
    fn clone(&self) -> Self {
        Self {
            building_block: self.building_block,
            has_ordered_trait: self.has_ordered_trait,
            rc: self.rc.clone(),
        }
    }
}

impl<'a, K, V, C> BuildingBlock<'a, K, V> for DynConcurrent<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        unsafe { self.building_block.as_ref().unwrap() }.capacity()
    }
    fn size(&self) -> usize {
        unsafe { self.building_block.as_ref().unwrap() }.size()
    }
    fn contains(&self, key: &K) -> bool {
        unsafe { self.building_block.as_ref().unwrap() }.contains(key)
    }
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        unsafe { self.building_block.as_mut().unwrap() }.take(key)
    }
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        unsafe { self.building_block.as_mut().unwrap() }
            .take_multiple(keys)
    }
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        unsafe { self.building_block.as_mut().unwrap() }.pop(n)
    }
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        unsafe { self.building_block.as_mut().unwrap() }.push(values)
    }
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        unsafe { self.building_block.as_mut().unwrap() }.flush()
    }
}

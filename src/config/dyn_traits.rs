use crate::config::ConfigError;
use crate::utils::lock::RWLock;
use crate::{BuildingBlock, Concurrent};

/// A Boxed [`BuildingBlock`] returned by a `ConfigBuilder`.
///
/// This object encapsulate a container built from a [configuration](index.html)
/// and whether it implements some of the crate top-level traits:
/// * [`Concurrent`](../trait.Concurrent.html)
///
/// Consequently, this object can be made into a similar
/// [`DynConcurrent`] object that will also implement the
/// [`Concurrent`](../trait.Concurrent.html) trait with the method
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent).
/// Of course, this dynamic trait implementation will only succeed if the
/// container that described by the initial configuration implements one of
/// these traits.
///
/// ## Examples
///
/// Making a valid configuration into a
/// [`BuildingBlock`](../trait.BuildingBlock.html) that carries the
/// [`Concurrent`](../trait.Concurrent.html) trait.
/// ```
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock, DynConcurrent};
///
/// // Write a `Concurrent` container configuration and make a `Builder`
/// // object from it.
/// let config = "
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ";
/// // The configuration is valid so we can `unwrap()`.
/// let builder = ConfigBuilder::from_string(config).unwrap();
///
/// // This configuration can be made into an `Concurrent` `BuildingBlock`.
/// let container: DynConcurrent<DynBuildingBlock<u64,u64>> =
///                builder.clone().build().into_concurrent().unwrap();
/// ```
///
/// Not all the configurations can support these traits.
/// ```
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder,
///                    ConfigError,
///                    DynBuildingBlock,
///                    DynConcurrent};
///
/// // Write a `Concurrent` container configuration and make a `Builder`
/// // object from it.
/// let config = "
/// id='BTreeConfig'
/// capacity=10
/// ";
/// // The configuration is valid so we can `unwrap()`.
/// let builder = ConfigBuilder::from_string(config).unwrap();
///
/// // This configuration cannot be made into an `Concurrent` `BuildingBlock`.
/// let result: Result<DynConcurrent<DynBuildingBlock<u64,u64>>, ConfigError> =
///          builder.clone().build().into_concurrent();
/// assert!(result.is_err());
/// ```
pub struct DynBuildingBlock<'a, K, V> {
    building_block: Box<dyn BuildingBlock<'a, K, V> + 'a>,
    has_concurrent_trait: bool,
}

impl<'a, K, V> DynBuildingBlock<'a, K, V> {
    pub(super) fn new(
        building_block: Box<dyn BuildingBlock<'a, K, V> + 'a>,
        has_concurrent_trait: bool,
    ) -> Self {
        Self {
            building_block,
            has_concurrent_trait,
        }
    }

    /// Make this [`BuildingBlock`](../trait.BuildingBlock.html) into an
    /// [`Concurrent`](../traits/trait.Concurrent.html)
    /// [`BuildingBlock`](../trait.BuildingBlock.html).
    /// If the initial configuration that was used to make this
    /// [`DynBuildingBlock`] did not support the
    /// [`Concurrent`](../trait.Concurrent.html) trait, a
    /// [`ConfigError::UnsupportedTraitError`] is returned, else,
    /// a [`BuildingBlock`](../trait.BuildingBlock.html) that carries
    /// the [`Concurrent`](../trait.Concurrent.html) trait is returned.
    pub fn into_concurrent(
        self,
    ) -> Result<DynConcurrent<Self>, ConfigError> {
        if self.has_concurrent_trait {
            Ok(DynConcurrent::new(self))
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

/// A wrapper around a `DynBuildingBlock` that provides the `Concurrent` trait.
///
/// This object is obtained from building a [config](index.html)
/// [`Builder`](struct.Builder.html) object into a [`DynBuildingBlock`]
/// and further calling the
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent)
/// method of the [`DynBuildingBlock`] object.
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
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// // Build a container from a configuration.
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string("
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ").unwrap().build();
///
/// // Sequential containers are `Concurrent` so it is ok to call the
/// // `into_concurrent()` method.
/// let container = container.into_concurrent().unwrap();
/// ```
pub struct DynConcurrent<C> {
    building_block: *mut C,
    rc: RWLock,
}

impl<C> DynConcurrent<C> {
    pub(crate) fn new(bb: C) -> Self {
        let rc = RWLock::new();
        rc.lock().unwrap();
        let bb = Box::into_raw(Box::new(bb));
        Self {
            building_block: bb,
            rc,
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

unsafe impl<C> Send for DynConcurrent<C> {}

unsafe impl<C> Sync for DynConcurrent<C> {}

impl<C> Concurrent for DynConcurrent<C> {
    fn clone(&self) -> Self {
        Self {
            building_block: self.building_block,
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

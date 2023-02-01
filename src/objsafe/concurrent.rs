use crate::utils::lock::RWLock;
use crate::{BuildingBlock, Concurrent};

/// A wrapper around a [`DynBuildingBlock`](struct.DynBuildingBlock.html) that
/// provides the `Concurrent` trait.
///
/// This object can only be constructed from a
/// [`DynBuildingBlock`](struct.DynBuildingBlock.html) using the latter
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent)
/// method.
///
/// Under the hood, this structure wraps a reference counter on a pointer to
/// a [`DynBuildingBlock`](struct.DynBuildingBlock.html) object. When the
/// [`Concurrent::clone()`](../trait.Concurrent.html#tymethod.clone) method
/// is called, the reference count is incremented and the pointer copied.
/// When the last clone goes out of scope, the pointer is freed.
/// This can only work safely if the pointer points to a building block that
/// can effectively be safely used concurrently without needing to check
/// borrowing rules. This is supposedly checked by the object implementing
/// [`std::convert::From`] creating the
/// [`DynBuildingBlock`](struct.DynBuildingBlock.html).
pub struct DynConcurrent<C> {
    building_block: *mut C,
    rc: RWLock,
}

impl<C> DynConcurrent<C> {
    pub(super) fn new(bb: C) -> Self {
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

impl<K, V, C> BuildingBlock<K, V> for DynConcurrent<C>
where
    C: BuildingBlock<K, V>,
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

    type FlushIterator = C::FlushIterator;
    fn flush(&mut self) -> Self::FlushIterator {
        unsafe { self.building_block.as_mut().unwrap() }.flush()
    }
}

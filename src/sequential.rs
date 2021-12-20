use crate::private::clone::CloneCell;
use crate::private::lock::{LockError, RWLock};
use crate::{BuildingBlock, Concurrent, Get, GetMut, Ordered, Prefetch};
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Sequential wrapper implementation                                      //
//------------------------------------------------------------------------//

/// Concurrent building block wrapper with a lock.
///
/// Makes a container thread safe by sequentializing its access.
///
/// # Examples
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
/// assert_eq!(key, 1u16);
/// assert_eq!(value, 12);
///```
pub struct Sequential<C> {
    container: CloneCell<C>,
    lock: RWLock,
}

impl<C> Sequential<C> {
    /// Construct a new concurrent container wrapping an existing
    /// `container`.
    pub fn new(container: C) -> Self {
        Sequential {
            container: CloneCell::new(container),
            lock: RWLock::new(),
        }
    }

    /// Get mutable access to a wrapped container.
    ///
    /// # Safety
    ///
    /// Lock is not acquired.
    /// Therefore, the use of returned container
    /// is not thread safe. Management of thread safety
    /// is left to the carefull user.
    pub unsafe fn deref_mut(&mut self) -> &mut C {
        &mut *self.container
    }

    /// Lock the container for exclusive access.
    pub fn lock_mut(&self) -> Result<(), LockError<()>> {
        self.lock.lock_mut()
    }

    /// Unlock the container.
    pub fn unlock(&self) {
        self.lock.unlock()
    }
}

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.lock.lock().unwrap();
        let out = Box::new(self.container.flush());
        self.lock.unlock();
        out
    }

    fn count(&self) -> usize {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.count()
    }

    fn contains(&self, key: &K) -> bool {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.contains(key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.take(key)
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.pop(n)
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        match self.lock.lock_mut() {
            Ok(_) => {
                let out = self.container.push(elements);
                self.lock.unlock();
                out
            }
            Err(_) => Vec::new(),
        }
    }
}

impl<V: Ord, C> Ordered<V> for Sequential<C> where C: Ordered<V> {}

//------------------------------------------------------------------------//
// Concurrent trait implementation                                        //
//------------------------------------------------------------------------//

unsafe impl<C> Send for Sequential<C> {}

unsafe impl<C> Sync for Sequential<C> {}

impl<C> Concurrent for Sequential<C> {
    fn clone(&self) -> Self {
        Sequential {
            container: self.container.clone(),
            lock: self.lock.clone(),
        }
    }
}

//------------------------------------------------------------------------//
// Get Trait Implementation                                               //
//------------------------------------------------------------------------//

/// Element from a building block wrapped in a `Sequential` building block.
///
/// This structure holds both the element and a lock on the container
/// where the element comes from. The lock is either shared or exclusive
/// depending on whether the element is read-only or read-write.
///
/// # Safety:
///
/// While this structure will prevent unsafe access between the
/// values and the building block containing them, if an unsafe access to
/// the container is attempted, while values wrapped in this struct are
/// alive, the caller will be stalled waiting to acquire the lock, and
/// potentially making a deadlock.
pub struct SequentialCell<V> {
    value: V,
    lock: RWLock,
}

impl<V> SequentialCell<V> {
    pub fn new(value: V, lock: &RWLock) -> Self {
        SequentialCell {
            value,
            lock: lock.clone(),
        }
    }
}

impl<V> Drop for SequentialCell<V> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

impl<V, W> Deref for SequentialCell<V>
where
    V: Deref<Target = W>,
{
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<V, W> DerefMut for SequentialCell<V>
where
    V: DerefMut<Target = W>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}

impl<K, V, U, C> Get<K, V, SequentialCell<U>> for Sequential<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<SequentialCell<U>> {
        match self.lock.lock() {
            Ok(_) => match (*self.container).get(key) {
                None => {
                    self.lock.unlock();
                    None
                }
                Some(w) => Some(SequentialCell::new(w, &self.lock)),
            },
            Err(_) => None,
        }
    }
}

impl<K, V, W, C> GetMut<K, V, SequentialCell<W>> for Sequential<C>
where
    W: DerefMut<Target = V>,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<SequentialCell<W>> {
        match self.lock.lock_mut() {
            Ok(_) => match (*self.container).get_mut(key) {
                None => {
                    self.lock.unlock();
                    None
                }
                Some(w) => Some(SequentialCell::new(w, &self.lock)),
            },
            Err(_) => None,
        }
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C> Prefetch<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.prefetch(keys)
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.take_multiple(keys)
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::tests::{
        test_building_block, test_concurrent, test_get, test_get_mut,
        test_prefetch,
    };
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Sequential::new(Array::new(0)));
        test_building_block(Sequential::new(Array::new(100)));
    }

    #[test]
    fn concurrent() {
        test_concurrent(Sequential::new(Array::new(0)), 64);
        test_concurrent(Sequential::new(Array::new(100)), 64);
    }

    #[test]
    fn get() {
        test_get(Sequential::new(Array::new(0)));
        test_get(Sequential::new(Array::new(100)));
        test_get_mut(Sequential::new(Array::new(0)));
        test_get_mut(Sequential::new(Array::new(100)));
    }

    #[test]
    fn prefetch() {
        test_prefetch(Sequential::new(Array::new(0)));
        test_prefetch(Sequential::new(Array::new(100)));
    }
}

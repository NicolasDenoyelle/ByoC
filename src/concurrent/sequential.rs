use crate::concurrent::Concurrent;
use crate::private::clone::CloneCell;
use crate::private::lock::{LockError, RWLock};
use crate::{BuildingBlock, Get};
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Sequential wrapper implementation                                      //
//------------------------------------------------------------------------//

/// Concurrent [`BuildingBlock`](../trait.BuildingBlock.html) wrapper with a lock.
/// Makes a container thread safe by sequentializing its access.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::concurrent::Concurrent;
/// use cache::container::Vector;
/// use cache::concurrent::Sequential;
///
/// // Build a concurrent Vector cache.
/// let mut c1 = Sequential::new(Vector::new(1));
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

    /// Get mutable access to wrapped container.
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

//------------------------------------------------------------------------//
// Concurrent trait implementation                                        //
//------------------------------------------------------------------------//

unsafe impl<C> Send for Sequential<C> {}

unsafe impl<C> Sync for Sequential<C> {}

impl<'a, K, V, C> Concurrent<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
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

pub struct LockedItem<V> {
    value: V,
    lock: RWLock,
}

impl<V> LockedItem<V> {
    pub fn new(value: V, lock: &RWLock) -> Self {
        LockedItem {
            value: value,
            lock: lock.clone(),
        }
    }
}

impl<V> Drop for LockedItem<V> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

impl<V, W> Deref for LockedItem<V>
where
    V: Deref<Target = W>,
{
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<V, W> DerefMut for LockedItem<V>
where
    V: DerefMut<Target = W>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}

impl<K, V, U, W, C> Get<K, V, LockedItem<U>, LockedItem<W>>
    for Sequential<C>
where
    U: Deref<Target = V>,
    W: DerefMut<Target = V>,
    C: Get<K, V, U, W>,
{
    unsafe fn get(&self, key: &K) -> Option<LockedItem<U>> {
        match self.lock.lock() {
            Ok(_) => match (*self.container).get(key) {
                None => {
                    self.lock.unlock();
                    None
                }
                Some(w) => Some(LockedItem::new(w, &self.lock)),
            },
            Err(_) => None,
        }
    }

    unsafe fn get_mut(&mut self, key: &K) -> Option<LockedItem<W>> {
        match self.lock.lock_mut() {
            Ok(_) => match (*self.container).get_mut(key) {
                None => {
                    self.lock.unlock();
                    None
                }
                Some(w) => Some(LockedItem::new(w, &self.lock)),
            },
            Err(_) => None,
        }
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::concurrent::tests::test_concurrent;
    use crate::container::Vector;
    use crate::tests::{test_building_block, test_get};

    #[test]
    fn building_block() {
        test_building_block(Sequential::new(Vector::new(0)));
        test_building_block(Sequential::new(Vector::new(100)));
    }

    #[test]
    fn concurrent() {
        test_concurrent(Sequential::new(Vector::new(0)), 64);
        test_concurrent(Sequential::new(Vector::new(100)), 64);
    }

    #[test]
    fn get() {
        test_get(Sequential::new(Vector::new(0)));
        test_get(Sequential::new(Vector::new(100)));
    }
}

use crate::private::clone::CloneCell;
use crate::private::lock::{LockError, RWLock, RWLockGuard};
use crate::{building_block::Concurrent, BuildingBlock, Get};
use std::marker::Sync;

//------------------------------------------------------------------------//
// Concurrent cache                                                       //
//------------------------------------------------------------------------//

/// Concurrent [`BuildingBlock`](../trait.BuildingBlock.html) wrapper with a lock.
/// Makes a container thread safe by sequentializing its access.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::building_block::Concurrent;
/// use cache::building_block::container::Vector;
/// use cache::building_block::wrapper::Sequential;
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

    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
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

impl<'a, K, V, C> Get<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V> + Get<'a, K, V>,
{
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b> {
        match self.lock.lock() {
            Ok(_) => Box::new(RWLockGuard::new(
                &self.lock,
                (*self.container).get(key),
            )),
            Err(_) => Box::new(std::iter::empty()),
        }
    }

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b> {
        match self.lock.lock_mut() {
            Ok(_) => Box::new(RWLockGuard::new(
                &self.lock,
                (*self.container).get_mut(key),
            )),
            Err(_) => Box::new(std::iter::empty()),
        }
    }
}

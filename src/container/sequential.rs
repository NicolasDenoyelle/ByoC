use crate::container::{Container, Get};
use crate::lock::{RWLock, RWLockGuard};
use crate::marker::{Concurrent, Packed};
use crate::utils::clone::CloneCell;
use std::marker::Sync;

//------------------------------------------------------------------------//
// Concurrent cache                                                       //
//------------------------------------------------------------------------//

/// Concurrent [`container`](trait.Container.html) wrapper with a lock.
/// Makes a container thread safe by sequentializing its access.
///
/// ## Generics:
///
/// * `C`: A type of [Container](trait.Container.html).
///
/// ## Examples
///
/// ```
/// use cache::marker::Concurrent;
/// use cache::container::{Container, Vector, Sequential};
///
/// // Build a concurrent Vector cache.
/// let mut c1 = Sequential::new(Vector::new(1));
/// let mut c2 = c1.clone();
///
/// assert!(c1.push(0u16, 4).is_none());
/// let (key, value) = c2.push(1u16, 12).unwrap();
/// assert_eq!(key, 0u16);
/// assert_eq!(value, 4);
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
    pub fn lock_mut(&self) {
        self.lock.lock_mut().unwrap()
    }

    /// Unlock the container.
    pub fn unlock(&self) {
        self.lock.unlock()
    }
}

impl<'a, K, V, C> Container<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Container<'a, K, V>,
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

    fn clear(&mut self) {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.clear()
    }

    fn take(&mut self, key: &K) -> Option<V> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.take(key)
    }

    fn pop(&mut self) -> Option<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.pop()
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.container.push(key, reference)
    }
}

impl<C> Clone for Sequential<C> {
    fn clone(&self) -> Self {
        Sequential {
            container: self.container.clone(),
            lock: self.lock.clone(),
        }
    }
}

impl<'a, K, V, C> Packed<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Container<'a, K, V> + Packed<'a, K, V>,
{
}

unsafe impl<C> Send for Sequential<C> {}

unsafe impl<C> Sync for Sequential<C> {}

impl<'a, K, V, C> Concurrent<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Container<'a, K, V>,
{
}

impl<'a, K, V, C, T> Get<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Get<'a, K, V, Item = T>,
    T: 'a,
{
    type Item = RWLockGuard<'a, T>;
    fn get(&'a mut self, key: &K) -> Option<RWLockGuard<T>> {
        self.lock_mut();
        match self.container.get(key) {
            None => None,
            Some(v) => Some(RWLockGuard::new(&self.lock, v)),
        }
    }
}

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
/// use cache::container::{Container, Concurrent, Map, Sequential};
/// use cache::reference::Default;
///
/// // Build a Map cache of 2 sets. Each set hold one element.
/// let mut c = Sequential::<_,Default<_>,_>::new(Map::new(1));
///
/// // Container as room for first and second element and returns None.
/// assert!(c.push(0u16, 4).is_none());
/// assert!(c.push(1u16, 12).is_some());
/// assert!(c.get(&0u16).is_some());
/// assert!(c.get(&1u16).is_none());
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

impl<K, V, C> Container<K, V> for Sequential<C>
where
    V: Ord,
    C: Container<K, V>,
{
    fn capacity(&self) -> usize {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.capacity()
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        self.lock.lock().unwrap();
        let v = self.container.flush();
        self.lock.unlock();
        v
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

impl<C: Clone> Clone for Sequential<C> {
    fn clone(&self) -> Self {
        Sequential {
            container: self.container.clone(),
            lock: self.lock.clone(),
        }
    }
}

impl<K, V, C> Packed<K, V> for Sequential<C>
where
    V: Ord,
    C: Container<K, V> + Packed<K, V>,
{
}

unsafe impl<C> Send for Sequential<C> {}

unsafe impl<C> Sync for Sequential<C> {}

impl<K, V, C> Concurrent<K, V> for Sequential<C>
where
    V: Ord,
    C: Container<K, V> + Clone,
{
}

impl<'a, K, V, C, T> Get<'a, K, V> for Sequential<C>
where
    V: Ord,
    C: Container<K, V> + Get<'a, K, V, Item = T>,
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

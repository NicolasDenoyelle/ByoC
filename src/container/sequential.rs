use crate::container::{Concurrent, Container, Get, Insert, Packed};
use crate::lock::{RWLock, RWLockGuard};
use crate::reference::{FromValue, Reference};
use crate::utils::clone::CloneCell;
use std::marker::{PhantomData, Sync};

//---------------------------------------------------------------------------//
// Concurrent cache                                                          //
//---------------------------------------------------------------------------//

/// Concurrent [`container`](trait.Container.html) wrapper with a lock.
/// Makes a container thread safe by sequentializing its access.
///
/// ## Generics:
///
/// * `K`: The type of key to use.
/// trait to compute the set index from key.
/// * `V`: Value type stored in [cache reference](../reference/trait.Reference.html).
/// * `R`: A type of cache [reference](../reference/trait.Reference.html).
/// * `C`: A type of [Container](trait.Container.html).
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Concurrent, Insert, Map, Sequential};
/// use cache::reference::Default;
///
/// // Build a Map cache of 2 sets. Each set hold one element.
/// let mut c = Sequential::<_,Default<_>,_>::new(Map::new(1));
///
/// // Container as room for first and second element and returns None.
/// assert!(c.insert(0u16, 4).is_none());
/// assert!(c.insert(1u16, 12).is_some());
/// assert!(c.get(&0u16).is_some());
/// assert!(c.get(&1u16).is_none());
///```
pub struct Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V>,
{
    container: CloneCell<C>,
    lock: RWLock,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V, C> Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V>,
{
    /// Construct a new concurrent container from a list of containers.
    ///
    /// The resulting concurrent container will have as many sets as containers in
    /// input.
    ///
    /// * `n_sets`: The number of sets for this container.
    /// * `set_size`: The capacity of each set. Every set of this container have
    /// the same capacity.
    /// * `new`: A container constructor closure taking the set size as argument
    /// to build a container of the same capacity.
    pub fn new(container: C) -> Self {
        Sequential {
            container: CloneCell::new(container),
            lock: RWLock::new(),
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }

    /// Get access to wrapped container.
    /// Lock is not acquired.
    /// Therefore, the use of returned container
    /// is not thread safe. Management of thread safety
    /// is left to the carefull user.
    pub unsafe fn deref(&self) -> &C {
        &*self.container
    }

    /// Get mutable access to wrapped container.
    /// Lock is not acquired.
    /// Therefore, the use of returned container
    /// is not thread safe. Management of thread safety
    /// is left to the carefull user.
    pub unsafe fn deref_mut(&mut self) -> &mut C {
        &mut *self.container
    }

    /// Lock the container for shared access.
    pub fn lock(&self) {
        self.lock.lock().unwrap()
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

impl<K, V, C> Container<K, V> for Sequential<K, V, C>
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

impl<K, V, C> Clone for Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V>,
{
    fn clone(&self) -> Self {
        Sequential {
            container: self.container.clone(),
            lock: self.lock.clone(),
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }
}

impl<K, V, C> Packed<K, V> for Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V> + Packed<K, V>,
{
}

impl<K, V, R, C> Insert<K, V, R> for Sequential<K, R, C>
where
    R: Reference<V> + FromValue<V>,
    C: Container<K, R>,
{
}

unsafe impl<K, V, C> Send for Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V>,
{
}

unsafe impl<K, V, C> Sync for Sequential<K, V, C>
where
    V: Ord,
    C: Container<K, V>,
{
}

impl<K, V, R, C> Concurrent<K, V, R> for Sequential<K, R, C>
where
    R: Reference<V>,
    C: Container<K, R> + Get<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        match self.container.get(key) {
            None => None,
            Some(v) => Some(RWLockGuard::new(&self.lock, v)),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>> {
        self.lock.lock_mut_for(()).unwrap();
        match self.container.get_mut(key) {
            None => {
                self.lock.unlock();
                None
            }
            Some(v) => Some(RWLockGuard::new(&self.lock, v)),
        }
    }
}

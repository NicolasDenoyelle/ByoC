use crate::container::{
    Concurrent, Container, Insert, Iter, IterMut, Packed, Sequential as Seq,
};
use crate::lock::{RWLock, RWLockGuard};
use crate::reference::{FromValue, Reference};
use std::marker::{PhantomData, Sync};

//----------------------------------------------------------------------------//
// Concurrent cache                                                           //
//----------------------------------------------------------------------------//

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
/// use cache::container::{Container, Concurrent, Insert};
/// use cache::container::sequential::Map;
/// use cache::container::concurrent::Sequential;
/// use cache::reference::Default;
///
/// // Build a Map cache of 2 sets. Each set hold one element.
/// let mut c = Sequential::<_,_,Default<_>,_>::new(Map::new(1));
///
/// // Container as room for first and second element and returns None.
/// assert!(c.insert(0u16, 4).is_none());
/// assert!(c.insert(1u16, 12).is_some());
/// assert!(c.get(&0u16).is_some());
/// assert!(c.get(&1u16).is_none());
///```
pub struct Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    container: C,
    lock: RWLock,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
    unused_r: PhantomData<R>,
}

impl<K, V, R, C> Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R>,
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
            container: container,
            lock: RWLock::new(),
            unused_k: PhantomData,
            unused_v: PhantomData,
            unused_r: PhantomData,
        }
    }

    /// Get access to wrapped container.
    /// Lock is not acquired.
    /// Therefore, the use of returned container
    /// is not thread safe. Management of thread safety
    /// is left to the carefull user.
    pub unsafe fn deref(&self) -> &C {
        &self.container
    }

    /// Get mutable access to wrapped container.
    /// Lock is not acquired.
    /// Therefore, the use of returned container
    /// is not thread safe. Management of thread safety
    /// is left to the carefull user.
    pub unsafe fn deref_mut(&mut self) -> &mut C {
        &mut self.container
    }

    /// Unwrap concurrent sequential and return wrapped container.
    pub fn unwrap(self) -> C {
        self.lock.lock_mut();
        self.container
    }

    /// Lock the container for shared access.
    pub fn lock(&self) {
        self.lock.lock()
    }

    /// Lock the container for exclusive access.
    pub fn lock_mut(&self) {
        self.lock.lock_mut()
    }

    /// Lock the container for shared access.
    pub fn try_lock(&self) -> bool {
        self.lock.try_lock()
    }

    /// Lock the container for exclusive access.
    pub fn try_lock_mut(&self) -> bool {
        self.lock.try_lock_mut()
    }

    /// Unlock the container.
    pub fn unlock(&self) {
        self.lock.unlock()
    }
}

impl<K, V, R, C> Container<K, V, R> for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    fn capacity(&self) -> usize {
        self.lock.lock();
        let c = self.container.capacity();
        self.lock.unlock();
        c
    }

    fn count(&self) -> usize {
        self.lock.lock();
        let c = self.container.count();
        self.lock.unlock();
        c
    }

    fn contains(&self, key: &K) -> bool {
        self.lock.lock();
        let c = self.container.contains(key);
        self.lock.unlock();
        c
    }

    fn clear(&mut self) {
        self.lock.lock_mut();
        self.container.clear();
        self.lock.unlock();
    }

    fn take(&mut self, key: &K) -> Option<R> {
        self.lock.lock_mut();
        let ret = self.container.take(key);
        self.lock.unlock();
        ret
    }

    fn pop(&mut self) -> Option<(K, R)> {
        self.lock.lock_mut();
        let v = self.container.pop();
        self.lock.unlock();
        v
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        self.lock.lock_mut();
        let v = self.container.push(key, reference);
        self.lock.unlock();
        v
    }
}

impl<K, V, R, C> Packed<K, V, R> for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R> + Packed<K, V, R>,
{
}

impl<K, V, R, C> Insert<K, V, R> for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V> + FromValue<V>,
    C: Container<K, V, R>,
{
}

unsafe impl<K, V, R, C> Send for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R>,
{
}

unsafe impl<K, V, R, C> Sync for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R>,
{
}

impl<K, V, R, C> Concurrent<K, V, R> for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R> + Seq<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>> {
        self.lock.lock_mut();
        match self.container.get(key) {
            None => {
                self.lock.unlock();
                None
            }
            Some(v) => Some(RWLockGuard::new(&self.lock, v)),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>> {
        self.lock.lock_mut();
        match self.container.get_mut(key) {
            None => {
                self.lock.unlock();
                None
            }
            Some(v) => Some(RWLockGuard::new(&self.lock, v)),
        }
    }
}

//----------------------------------------------------------------------------//
// iterator for concurrent cache                                              //
//----------------------------------------------------------------------------//

impl<'a, K, V, R, C, I> IntoIterator for Sequential<K, V, R, C>
where
    K: Ord + Clone,
    R: Reference<V>,
    C: Container<K, V, R> + IntoIterator<Item = (K, V), IntoIter = I>,
    I: Iterator<Item = (K, V)>,
{
    type Item = (K, V);
    type IntoIter = I;
    fn into_iter(self) -> Self::IntoIter {
        self.lock.lock_mut();
        self.container.into_iter()
    }
}

pub struct SequentialIter<'a, I: Iterator> {
    it: I,
    _guard: RWLockGuard<'a, bool>,
}

impl<'a, I: Iterator> Iterator for SequentialIter<'a, I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

impl<'a, K, V, R, C, I> Iter<'a, K, V, R> for Sequential<K, V, R, C>
where
    K: 'a + Ord + Clone,
    V: 'a,
    R: 'a + Reference<V>,
    C: Container<K, V, R> + Iter<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a V)>,
{
    type Iterator = SequentialIter<'a, I>;

    fn iter(&'a mut self) -> Self::Iterator {
        self.lock.lock_mut();
        SequentialIter {
            it: self.container.iter(),
            _guard: RWLockGuard::new(&self.lock, true),
        }
    }
}

impl<'a, K, V, R, C, I> IterMut<'a, K, V, R> for Sequential<K, V, R, C>
where
    K: 'a + Ord + Clone,
    V: 'a,
    R: 'a + Reference<V>,
    C: Container<K, V, R> + IterMut<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
{
    type Iterator = SequentialIter<'a, I>;

    fn iter_mut(&'a mut self) -> Self::Iterator {
        self.lock.lock_mut();
        SequentialIter {
            it: self.container.iter_mut(),
            _guard: RWLockGuard::new(&self.lock, true),
        }
    }
}

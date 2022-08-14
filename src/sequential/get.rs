use super::Sequential;
use crate::internal::lock::RWLock;
use crate::{Get, GetMut};
use std::ops::{Deref, DerefMut};

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
            Ok(_) => match self.container.as_ref().get(key) {
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
            Ok(_) => match self.container.as_mut().get_mut(key) {
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

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        test_get(Sequential::new(Array::new(0)));
        test_get(Sequential::new(Array::new(100)));
        test_get_mut(Sequential::new(Array::new(0)));
        test_get_mut(Sequential::new(Array::new(100)));
    }
}

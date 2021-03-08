use crate::container::{Concurrent, Container, Iter, IterMut, Sequential};
use crate::lock::{RWLock, RWLockGuard};
use crate::reference::Reference;
use std::boxed::Box;
use std::marker::Sync;
use std::ops::{Deref, DerefMut, Drop};

//------------------------------------------------------------------------------------//
//                      Ref counted cell inside clonable struct                       //
//------------------------------------------------------------------------------------//

struct InnerClone<V> {
    value: V,
    rc: RWLock,
}

impl<V> Deref for InnerClone<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for InnerClone<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<V> InnerClone<V> {
    pub fn new(value: V) -> Self {
        let rc = RWLock::new();
        // Increment reference count in the lock by one.
        rc.lock().unwrap();
        InnerClone {
            value: value,
            rc: rc,
        }
    }

    pub fn clone(&self) {
        self.rc.lock().unwrap();
    }

    pub fn drop(&mut self) -> bool {
        self.rc.unlock(); // release (last?) read lock.
        match self.rc.try_lock_mut() {
            Ok(_) => true,
            _ => false,
        } // Return whether we are the only remaining clone owner.
    }
}

//------------------------------------------------------------------------------------//
//                                 Public clonable struct                             //
//------------------------------------------------------------------------------------//

/// CloneCell is a generic wrapper to break the mutability rules.
/// CloneCell stores a raw pointer to its content and copies the pointer
/// on call to `clone()`. CloneCell keeps track of the count of clones
/// inside a [RWLock](../lock/struct.RWLock.html) and destroyes its content
/// when all the clones have gone out of scope.
/// CloneCell implements the [Containers](../container/trait.Container.html)
/// when it wraps a container. This allows for instance to clone a concurrent
/// container and perform concurrent mutable access to it.
/// Content inside a CloneCell struct can be accessed via `Deref` and `DerefMut`
/// traits.
///
/// # Example
/// ```ignore
/// use cache::utils::clone::CloneCell;
///
/// let mut v = CloneCell::new(4u32);
/// assert!(*v == 4u32);
///
/// let v2 = v.clone();
/// assert!(*v2 == *v);
/// *v = 5u32;
/// assert!(*v2 == 5u32);
/// ```
pub struct CloneCell<V> {
    ptr: *mut InnerClone<V>,
}

impl<V> CloneCell<V> {
    /// CloneCell constructor.
    /// Wraps a value into a `InnerClone`.
    ///
    pub fn new(value: V) -> Self {
        CloneCell {
            ptr: Box::into_raw(Box::new(InnerClone::new(value))),
        }
    }
}

impl<V> Clone for CloneCell<V> {
    /// Acquire shared lock on the `InnerClone` then
    /// copy `InnerClone=` pointer.
    fn clone(&self) -> Self {
        unsafe { (*self.ptr).clone() }
        CloneCell {
            ptr: self.ptr.clone(),
        }
    }
}

impl<V> Drop for CloneCell<V> {
    /// Acquire exclusive ownership of the `InnerClone`, then
    /// destroy the CloneCell and its content.
    fn drop(&mut self) {
        if InnerClone::drop(unsafe { &mut *self.ptr }) {
            drop(unsafe { Box::from_raw(self.ptr) })
        }
    }
}

impl<V> Deref for CloneCell<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        unsafe { (*self.ptr).deref() }
    }
}

impl<V> DerefMut for CloneCell<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.ptr).deref_mut() }
    }
}

unsafe impl<V: Send> Send for CloneCell<V> {}
unsafe impl<V: Sync> Sync for CloneCell<V> {}

//------------------------------------------------------------------------------------//
//                                 Container implementation                           //
//------------------------------------------------------------------------------------//

impl<'a, K, V, R, C> Container<K, V, R> for CloneCell<C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    fn capacity(&self) -> usize {
        self.deref().capacity()
    }

    fn count(&self) -> usize {
        self.deref().count()
    }

    fn contains(&self, key: &K) -> bool {
        self.deref().contains(key)
    }

    fn take(&mut self, key: &K) -> Option<R> {
        self.deref_mut().take(key)
    }

    fn pop(&mut self) -> Option<(K, R)> {
        self.deref_mut().pop()
    }

    fn clear(&mut self) {
        self.deref_mut().clear()
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        self.deref_mut().push(key, reference)
    }
}

impl<'a, K, V, R, C> Concurrent<K, V, R> for CloneCell<C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Concurrent<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>> {
        unsafe { (*self.ptr).get(key) }
    }

    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>> {
        unsafe { (*self.ptr).get_mut(key) }
    }
}

impl<'a, K, V, R, C> Sequential<K, V, R> for CloneCell<C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Sequential<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        unsafe { (*self.ptr).get(key) }
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        unsafe { (*self.ptr).get_mut(key) }
    }
}

impl<'a, K, V, R, C, I> Iter<'a, K, V, R> for CloneCell<C>
where
    K: 'a + Ord,
    V: 'a,
    R: 'a + Reference<V>,
    C: Container<K, V, R> + Iter<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a V)>,
{
    type Iterator = I;
    fn iter(&'a mut self) -> I {
        unsafe { (*self.ptr).iter() }
    }
}

impl<'a, K, V, R, C, I> IterMut<'a, K, V, R> for CloneCell<C>
where
    K: 'a + Ord,
    V: 'a,
    R: 'a + Reference<V>,
    C: Container<K, V, R> + IterMut<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
{
    type Iterator = I;
    fn iter_mut(&'a mut self) -> I {
        unsafe { (*self.ptr).iter_mut() }
    }
}

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(tests)]
mod tests {
    use super::CloneCell;
    use std::thread;

    #[test]
    fn test_clone() {
        let v = CloneCell::new(96u32);

        for i in 0..64 {
            let mut vc = v.clone();
            thread::spawn(move || {
                *vc = i as u32;
            });
        }

        assert!(*v < 64u32);
    }
}

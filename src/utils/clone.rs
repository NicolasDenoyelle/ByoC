use crate::container::{Container, Get};
use crate::lock::RWLock;
use crate::marker::Concurrent;
use std::boxed::Box;
use std::marker::Sync;
use std::ops::{Deref, DerefMut, Drop};

//------------------------------------------------------------------------//
//                 Ref counted cell inside clonable struct                //
//------------------------------------------------------------------------//

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

//------------------------------------------------------------------------//
//                            Public clonable struct                      //
//------------------------------------------------------------------------//

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

//-------------------------------------------------------------------------
//                         Container implementation
//-------------------------------------------------------------------------

impl<'a, K, V, C> Container<'a, K, V> for CloneCell<C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V>,
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

    fn take(&mut self, key: &K) -> Option<V> {
        self.deref_mut().take(key)
    }

    fn pop(&mut self) -> Option<(K, V)> {
        self.deref_mut().pop()
    }

    fn clear(&mut self) {
        self.deref_mut().clear()
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        self.deref_mut().push(key, reference)
    }
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.deref_mut().flush()
    }
}

impl<'a, K, V, C> Concurrent<'a, K, V> for CloneCell<C>
where
    K: 'a,
    V: 'a,
    C: Concurrent<'a, K, V>,
{
}

impl<'a, K, V, T, C> Get<'a, K, V> for CloneCell<C>
where
    K: 'a,
    V: 'a,
    C: Get<'a, K, V, Item = T>,
    T: 'a,
{
    type Item = T;
    fn get(&'a mut self, key: &K) -> Option<T> {
        unsafe { (*self.ptr).get(key) }
    }
}

//-------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------

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

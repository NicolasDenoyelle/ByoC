use crate::internal::lock::{RWLock, RWLockGuard, TryLockError};
use crate::{BuildingBlock, Concurrent, Get, GetMut, Ordered};
use std::boxed::Box;
use std::marker::Sync;
use std::ops::{Deref, DerefMut, Drop};

//------------------------------------------------------------------------//
//                            Public clonable struct                      //
//------------------------------------------------------------------------//

/// Reference counted raw pointer.
struct UnsafeSharedPtr<V> {
    ptr: *mut V,
    rc: RWLock,
}

impl<V> UnsafeSharedPtr<V> {
    /// Gain read only access to the object.
    /// This function is unsafe because shadow copies of the underlying
    /// object may write the underlying object and lead to unexpected or
    /// inconsistent reads when the object is being accessed concurrently.
    pub unsafe fn as_ref(&self) -> &V {
        // SAFETY:
        // As long as the UnsafeSharedPtr exists, its pointer is valid.
        // It is freed only when the last copy of the pointed content is
        // dropped. The only way for it to be invalid would be that the
        // rc counter is invalid or a memory corruption.
        self.ptr.as_ref().unwrap()
    }

    /// Gain write access to the object.
    /// This function is unsafe because shadow copies of the underlying
    /// object may write the underlying object at the same time and lead to
    /// inconsistent modifications.
    pub unsafe fn as_mut(&mut self) -> &mut V {
        // SAFETY:
        // As long as the UnsafeSharedPtr exists, its pointer is valid.
        // It is freed only when the last copy of the pointed content is
        // dropped. The only way for it to be invalid would be that the
        // rc counter is invalid or a memory corruption.
        self.ptr.as_mut().unwrap()
    }
}

impl<V> From<V> for UnsafeSharedPtr<V> {
    /// UnsafeSharedPtr constructor.
    /// Wrap a value into a reference counting cell and move it on the heap.
    fn from(value: V) -> Self {
        let rc = RWLock::new();
        // Increment reference count in the lock by one.
        rc.lock().unwrap();

        UnsafeSharedPtr {
            ptr: Box::into_raw(Box::new(value)),
            rc,
        }
    }
}

impl<V> Clone for UnsafeSharedPtr<V> {
    /// Create a shadow copy of the same object pointed by the same pointer.
    /// This function safely increments the count of copies of this pointer.
    /// Then it creates a shadow copy of the same element pointed by the same
    /// pointer. Although creating a shadow copy is safe, using it is not
    /// because it breaks the borrowing and concurrency rules.
    fn clone(&self) -> Self {
        self.rc.lock().unwrap();
        UnsafeSharedPtr {
            ptr: self.ptr,
            rc: self.rc.clone(),
        }
    }
}

impl<V> Drop for UnsafeSharedPtr<V> {
    /// This method decrements the count of shadow copies.
    /// Then, it tries to acquire exclusive ownership over the reference counted
    /// pointer. If ownership is acquired, then it means that there we no other
    /// shadow copies of this element, and the content is destroyed.
    fn drop(&mut self) {
        // release (last?) read lock.
        self.rc.unlock();
        // If we are the only remaining clone owner we clean ourselves up.
        if let Ok(_) = self.rc.try_lock_mut() {
            drop(unsafe { Box::from_raw(self.ptr) })
        }
    }
}

//-------------------------------------------------------------------------
// A Shared ptr blocking on access.
//-------------------------------------------------------------------------

/// Reference counting wrapper to allow safe concurrent access to an object.
///
/// [`SharedPtr`] stores its content on the heap and can create shadow
/// copies, i.e copies of the pointer to the same content. When the last copy of
/// the pointer is dropped, the value pointed by the pointer is also destroyed
/// and the space on the heap is freed.
///
/// The provided accessor methods guarantee safe exclusive or shared access to
/// the underlying pointer. This is achieved by embedding a read-write lock in
/// the [`SharedPtr`] smart pointer. Mutable borrows will try to write-lock
/// the lock while shared borrows will try to read-lock the lock. Any failure
/// to lock the lock will deny the access to the underlying object. Success to
/// gain access to the underlying object will result in wrapping the latter in
/// a RAII guard structure, thus ensuring that the lock is released when the
/// guard goes out of scope. The RAII guard also contains a clone of the
/// [`SharedPtr`], also ensuring that the pointer to the heap object remains
/// valid while the guard is in scope.
///
/// ## Example
///
/// ```rust,ignore
/// use byoc::internal::SharedPtr;
///
/// let v = SharedPtr::from(4u32);
/// {
///     // Shared access to the pointer.
///     let shared_guard = v.as_ref();
///     assert!(*shared_guard == 4u32);
///     // Other shared access ok. Does not panic.
///     let _shared_guard2 = v.get().unwrap();
///     // Mutable access not ok.
///     assert!(v.clone().get_mut().is_err());
/// }
/// let mut v2 = v.clone();
/// *v2.get_mut().unwrap() += 1u32; // Ok
/// assert_eq!(*v2.as_ref(), 5u32);
/// ```
pub struct SharedPtr<V> {
    ptr: UnsafeSharedPtr<V>,
    lock: RWLock,
}

pub struct SharedPtrGuard<'a, V> {
    ptr: RWLockGuard<'a, UnsafeSharedPtr<V>>,
}

impl<'a, V> Deref for SharedPtrGuard<'a, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // This object has been created by successfully acquiring the (read)
        // lock of the SharedPtr holder. Therefore it cannot be modified
        // elsewhere while this guard is in scope.
        unsafe { self.ptr.as_ref() }
    }
}

pub struct SharedPtrGuardMut<'a, V> {
    ptr: RWLockGuard<'a, UnsafeSharedPtr<V>>,
}

impl<'a, V> Deref for SharedPtrGuardMut<'a, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // This object has been created by successfully acquiring the exclusive
        // (write) lock of the SharedPtr holder. Therefore it cannot be
        // read or modified elsewhere while this guard is in scope.
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, V> DerefMut for SharedPtrGuardMut<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // This object has been created by successfully acquiring the exclusive
        // (write) lock of the SharedPtr holder. Therefore it cannot be
        // read or modified elsewhere while this guard is in scope.
        unsafe { self.ptr.as_mut() }
    }
}

impl<V> SharedPtr<V> {
    /// Gain read-only access to the pointed object.
    /// This function succeeds if there is no other exclusive access
    /// of the underlying pointer going on.
    ///
    /// On success, this function returns a guard that can be dereferenced
    /// into a shared reference to the pointed object. The guard holds a
    /// locked read-write lock from the [`SharedPtr`] that created it.
    /// This lock is unlocked when the guard is dropped.
    ///
    /// The function fails and returns an error in the following cases:
    /// * The pointer is being accessed mutably somewhere else,
    /// * A thread holding a guard from a copy of this pointer panicked and
    /// poisoned the associated lock.
    pub fn get<'a>(
        &'a self,
    ) -> Result<SharedPtrGuard<'a, V>, TryLockError<()>> {
        match self.lock.try_lock() {
            Ok(_) => Ok(SharedPtrGuard {
                ptr: RWLockGuard::new(&self.lock, self.ptr.clone()),
            }),
            Err(e) => Err(e),
        }
    }

    /// Gain read-write access to the pointed object.
    /// This function succeeds if there is no other clone of the same pointer
    /// being accessed simultaneously.
    ///
    /// On success, this function returns a guard that can be dereferenced
    /// into an exclusive reference to the pointed object. The guard holds a
    /// locked read-write lock from the [`SharedPtr`] that created it.
    /// This lock is unlocked when the guard is dropped.
    ///
    /// The function fails and returns an error in the following cases:
    /// * The pointer is being accessed somewhere else,
    /// * A thread holding a guard from a copy of this pointer panicked and
    /// poisoned the associated lock.
    pub fn get_mut<'a>(
        &'a mut self,
    ) -> Result<SharedPtrGuardMut<'a, V>, TryLockError<()>> {
        match self.lock.try_lock_mut() {
            Ok(_) => Ok(SharedPtrGuardMut {
                ptr: RWLockGuard::new(&self.lock, self.ptr.clone()),
            }),
            Err(e) => Err(e),
        }
    }

    /// This method has the same effect as
    /// [`get_mut()`](struct.SharedPtr.html#method.get_mut) method except
    /// it will panic if it cannot acquire the lock on the pointer.
    pub fn as_mut<'a>(&'a mut self) -> SharedPtrGuardMut<'a, V> {
        match self.get_mut() {
            Ok(ptr) => ptr,
            Err(TryLockError::WouldBlock(_)) => panic!("Cannot borrow SharedPtr mutably while being borrowed already."),
	    Err(TryLockError::Poisoned(_)) =>panic!("Cannot borrow poisoned SharedPtr."),
        }
    }

    /// This method has the same effect as
    /// [`get()`](struct.SharedPtr.html#method.get) method except it will
    /// panic if it cannot acquire the lock on the pointer.
    pub fn as_ref<'a>(&'a self) -> SharedPtrGuard<'a, V> {
        match self.get() {
            Ok(ptr) => ptr,
            Err(TryLockError::WouldBlock(_)) => panic!("Cannot mutably borrow SharedPtr mutably while being borrowed already."),
	    Err(TryLockError::Poisoned(_)) =>panic!("Cannot borrow poisoned SharedPtr."),
        }
    }
}

impl<V> From<V> for SharedPtr<V> {
    /// SharedPtr constructor.
    /// Wrap a value into a reference counting cell and move it on the heap.
    fn from(value: V) -> Self {
        SharedPtr {
            ptr: UnsafeSharedPtr::from(value),
            lock: RWLock::new(),
        }
    }
}

impl<V> Clone for SharedPtr<V> {
    /// Create a shadow copy of the same object pointed by the same pointer.
    /// This function safely increments the count of copies of this pointer.
    /// Then it creates a shadow copy of the same element pointed by the same
    /// pointer. The pointed object will be destroyed only when the last of its
    /// pointers goes out of scope.
    fn clone(&self) -> Self {
        SharedPtr {
            ptr: self.ptr.clone(),
            lock: self.lock.clone(),
        }
    }
}

impl<V: std::fmt::Debug> std::fmt::Debug for SharedPtr<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Ok(ptr) => ptr.fmt(f),
            Err(_) => f.write_fmt(format_args!(
                "SharedPtr<{}>: unavailable",
                std::any::type_name::<V>()
            )),
        }
    }
}

impl<'a, V: std::fmt::Debug> std::fmt::Debug for SharedPtrGuard<'a, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, V: std::fmt::Debug> std::fmt::Debug for SharedPtrGuardMut<'a, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

unsafe impl<V: Send> Send for SharedPtr<V> {}
unsafe impl<V: Sync> Sync for SharedPtr<V> {}

impl<T, V: Iterator<Item = T>> Iterator for SharedPtr<V> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }
}
//-------------------------------------------------------------------------
//                         BuildingBlock implementation
//-------------------------------------------------------------------------

impl<'a, K, V, C> BuildingBlock<'a, K, V> for SharedPtr<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.as_ref().capacity()
    }

    fn count(&self) -> usize {
        self.as_ref().count()
    }

    fn contains(&self, key: &K) -> bool {
        self.as_ref().contains(key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.as_mut().take(key)
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.as_mut().pop(n)
    }

    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        self.as_mut().push(values)
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.as_mut().flush()
    }
}

impl<C: Send + Sync> Concurrent for SharedPtr<C> {
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<V: Ord, C: Ordered<V>> Ordered<V> for SharedPtr<C> {}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

impl<K, V, C, U> Get<K, V, U> for SharedPtr<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        self.as_ref().get(key)
    }
}

impl<K, V, C, W> GetMut<K, V, W> for SharedPtr<C>
where
    W: Deref<Target = V> + DerefMut,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
        self.as_mut().get_mut(key)
    }
}

//-------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::SharedPtr;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_concurrent() {
        let v = Arc::new(Mutex::new(SharedPtr::from(96u32)));

        for i in 0..64 {
            let vc = v.clone();
            thread::spawn(move || {
                let mut vc = vc.lock().unwrap();
                *(vc.as_mut()) = i as u32;
            });
        }

        assert!(*v.lock().unwrap().as_ref() < 64u32);
    }

    #[test]
    fn test_failed_access() {
        let mut ptr1 = SharedPtr::from(96u32);
        let mut ptr2 = ptr1.clone();

        {
            let i = ptr1.as_ref();
            assert_eq!(*i, *ptr2.as_ref());
            assert!(ptr2.get_mut().is_err());
        }

        {
            let mut i = ptr1.as_mut();
            *i += 1;
            assert!(ptr2.get().is_err());
        }
    }

    #[test]
    fn test_doc() {
        let v = SharedPtr::from(4u32);

        {
            // Shared access to the pointer.
            let shared_guard = v.as_ref();
            assert!(*shared_guard == 4u32);

            // Other shared access ok. Does not panic.
            let _shared_guard2 = v.get().unwrap();

            // Mutable access not ok.
            assert!(v.clone().get_mut().is_err());
        }

        let mut v2 = v;
        *v2.get_mut().unwrap() += 1u32; // Ok
        assert_eq!(*v2.as_ref(), 5u32);
    }
}

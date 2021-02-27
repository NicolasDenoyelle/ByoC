use std::cmp::max;
use std::marker::Sync;
use std::ops::Drop;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{thread, time};

static EXCLUSIVE: u64 =
    0b1000000000000000000000000000000000000000000000000000000000000000;

/// A custom Read Write Lock implementation based on Rust Atomic primitives.
/// Unlike Rust RWLock interface, this RWLock allow to call `unlock()` after `lock()`.
/// This comes with a [RWLockGuard](struct.RWLockGuard.html) companion, that allows
/// creation of objects that call `unlock()` on a RWLock when they go out of scope.
///
/// # Examples
///
/// ```ignore
/// use cache::utils::lock::RWLock;
/// let lock = RWLock::new();
///
/// // exclusive lock
/// lock.lock_mut();
/// assert!(!lock.try_lock());
/// assert!(!lock.try_lock_mut());
/// lock.unlock();
///
/// // shared lock
/// assert!(lock.try_lock());
/// assert!(lock.try_lock());
/// assert!(!lock.try_lock_mut());
/// lock.unlock();
/// assert!(!lock.try_lock_mut());
/// lock.unlock();
/// assert!(lock.try_lock_mut());
/// ```
#[derive(Debug)]
pub struct RWLock {
    // Unlocked
    // [ 0000000000000000000000000000000000000000000000000000000000000000 ]
    // EXCLUSIVE
    // [ 1000000000000000000000000000000000000000000000000000000000000000 ]
    // Shared
    // [ 0XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX ]
    state: AtomicU64,
}

unsafe impl Send for RWLock {}
unsafe impl Sync for RWLock {}

impl RWLock {
    /// Construct a new lock.
    pub fn new() -> Self {
        RWLock {
            state: AtomicU64::new(0),
        }
    }

    /// Get the number of times this has been locked in shared state.
    /// It is mainly used for debug.
    pub fn weak_count(&self) -> u64 {
        self.state.load(Ordering::Relaxed)
    }

    /// Try to acquire shared access to the lock.
    /// Multiple `try_lock()` will succeed as long as the lock is unlocked,
    /// or locked in a shared state.
    /// Return true if the lock was acquired, else false.
    /// `try_lock()` may fail if the lock is locked in exclusive state or if
    /// a thread is currently performing operation on the lock.
    /// Call `unlock()` to unlock after succesfull `try_lock()`.
    pub fn try_lock(&self) -> bool {
        let count = self.state.load(Ordering::SeqCst);
        if (count & EXCLUSIVE) != 0 {
            return false;
        } else {
            match self.state.compare_exchange_weak(
                count,
                count + 1,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    /// Try to acquire exclusive access to the lock.
    /// `try_lock_mut()` will succeed only if the lock is unlocked.
    /// Return true if the lock was acquired, else false.
    /// `try_lock_mut()` may fail if the lock is already locked or if
    /// a thread is currently performing operation on the lock.    
    /// Call `unlock()` to unlock after succesfull `try_lock_mut()`.
    pub fn try_lock_mut(&self) -> bool {
        match self.state.compare_exchange_weak(
            0,
            EXCLUSIVE,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            Err(_) => false,
            Ok(_) => true,
        }
    }

    /// Hang until shared access to the lock is granted.
    /// Call `unlock()` to unlock after succesfull `lock()`.    
    pub fn lock(&self) {
        let mut nanos: u64 = 1;
        while !self.try_lock() {
            nanos = max(nanos * 2, 1000);
            thread::sleep(time::Duration::from_nanos(nanos));
        }
    }

    /// Hang until exclusive access to the lock is granted.
    /// Call `unlock()` to unlock after succesfull `lock_mut()`.    
    pub fn lock_mut(&self) {
        let mut nanos: u64 = 1;
        while !self.try_lock_mut() {
            nanos = max(nanos * 2, 1000);
            thread::sleep(time::Duration::from_nanos(nanos));
        }
    }

    /// Unlock a lock after it is acquired.
    /// If the lock is not locked, nothing happens.
    /// If the lock is locked in exclusive mode, unlock this mode, else,
    /// decrease the number of shared access acquisitions. If the count of shared
    /// access acquisition falls to 0, the lock is unlocked.
    pub fn unlock(&self) {
        let mut count = match self.state.compare_exchange_weak(
            EXCLUSIVE,
            0,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(c) => c,
            Err(c) => c,
        };

        loop {
            if count == 0 || count == EXCLUSIVE {
                break;
            }
            match self.state.compare_exchange_weak(
                count,
                count - 1,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => count = x,
            }
        }
    }
}

/// A RWLock guard that unlock the lock when going out of scope.
/// Element inside a RWLockGuard can be accessed with Deref and DerefMut methods.
///
/// # Examples
///
/// ```ignore
/// use cache::utils::lock::{RWLock, RWLockGuard};
/// let lock = RWLock::new();
/// lock.lock_mut();
///
/// {
///       let guard = RWLockGuard::new(&lock, 0);
///       assert_eq!(*guard, 0);
/// } // guard goes out of scope and lock gets unlocked.
///
/// assert!(lock.try_lock_mut());
/// ```
#[derive(Debug)]
pub struct RWLockGuard<'a, V> {
    lock: &'a RWLock,
    value: V,
}

impl<'a, V> RWLockGuard<'a, V> {
    /// Construct a new guard.
    pub fn new(l: &'a RWLock, v: V) -> Self {
        RWLockGuard { lock: l, value: v }
    }
}

/// Access inner value
impl<'a, V> Deref for RWLockGuard<'a, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Access inner value
impl<'a, V> DerefMut for RWLockGuard<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// Unlock embedded lock.
impl<'a, V> Drop for RWLockGuard<'a, V> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

#[derive(Debug)]
pub struct RWLockWrapper<V> {
    lock: RWLock,
    value: V,
}

#[allow(dead_code)]
impl<V> RWLockWrapper<V> {
    /// Construct a new lock wrapping a value.
    pub fn new(v: V) -> Self {
        RWLockWrapper {
            lock: RWLock::new(),
            value: v,
        }
    }

    /// Try to acquire shared access to the wrapped value.
    /// Return None on failure, Some lock guard around the value on success.
    pub fn try_lock(&self) -> Option<RWLockGuard<&V>> {
        if self.lock.try_lock() {
            Some(RWLockGuard::new(&self.lock, &self.value))
        } else {
            None
        }
    }

    /// Try to acquire exclusive access to the wrapped value.
    /// Return None on failure, Some lock guard around the value on success.
    pub fn try_lock_mut(&mut self) -> Option<RWLockGuard<&mut V>> {
        if self.lock.try_lock_mut() {
            Some(RWLockGuard::new(&self.lock, &mut self.value))
        } else {
            None
        }
    }

    /// Hang until shared access to the wrapped balue is granted.
    pub fn lock(&self) -> RWLockGuard<&V> {
        self.lock.lock();
        RWLockGuard::new(&self.lock, &self.value)
    }

    /// Hang until exclusive access to the wrapped value is granted.
    pub fn lock_mut(&mut self) -> RWLockGuard<&mut V> {
        self.lock.lock_mut();
        RWLockGuard::new(&self.lock, &mut self.value)
    }
}

//------------------------------------------------------------------------------------//
//                                        Tests                                       //
//------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::RWLock;
    use std::sync::Arc;
    use std::thread;
    use std::thread::JoinHandle;

    #[test]
    fn test_lock() {
        let lock = Arc::new(RWLock::new());
        let num_threads = 1024;
        let mut threads: Vec<JoinHandle<_>> = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let local_lock = Arc::clone(&lock);
            threads.push(thread::spawn(move || {
                local_lock.lock();
                assert!(!local_lock.try_lock_mut());
            }));
        }

        while let Some(t) = threads.pop() {
            t.join().unwrap();
        }

        for _ in 0..num_threads {
            let local_lock = Arc::clone(&lock);
            threads.push(thread::spawn(move || {
                local_lock.unlock();
            }));
        }

        while let Some(t) = threads.pop() {
            t.join().unwrap();
        }

        assert!(lock.try_lock_mut());
    }
}

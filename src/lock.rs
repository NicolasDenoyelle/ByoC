use std::cmp::max;
use std::ops::Drop;
use std::ops::{Deref, DerefMut};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::{thread, time};

static EXCLUSIVE: u64 =
    0b0100000000000000000000000000000000000000000000000000000000000000;

static POISONED: u64 =
    0b1000000000000000000000000000000000000000000000000000000000000000;

#[derive(Debug)]
pub enum TryLockError<T> {
    WouldBlock(T),
    Poisoned(T),
}

#[derive(Debug)]
pub enum LockError<T> {
    Poisoned(T),
}

/// A custom Read Write Lock implementation based on Rust Atomic primitives.
/// Unlike Rust RWLock interface, this RWLock allow to call `unlock()`
/// after `lock()`.
/// This comes with a [RWLockGuard](struct.RWLockGuard.html) companion,
/// that allows creation of objects that call `unlock()` on a RWLock when
/// they go out of scope.
///
/// # Examples
/// ```ignore
/// use cache::lock::RWLock;
/// let lock = RWLock::new();
///
/// // exclusive lock
/// assert!(lock.lock_mut().is_ok());
/// assert!(lock.try_lock().is_err());
/// assert!(lock.try_lock_mut().is_err());
/// lock.unlock();
///
/// // shared lock
/// assert!(lock.try_lock().is_ok());
/// assert!(lock.try_lock().is_ok());
/// assert!(lock.try_lock_mut().is_err());
/// lock.unlock();
/// assert!(lock.try_lock_mut().is_err());
/// lock.unlock();
/// assert!(lock.try_lock_mut().is_ok());
/// ```
#[derive(Debug)]
pub struct RWLock {
    // Unlocked
    // [ 0000000000000000000000000000000000000000000000000000000000000000 ]
    // Exclusive
    // [ 0100000000000000000000000000000000000000000000000000000000000000 ]
    // Poisoned
    // [ 1000000000000000000000000000000000000000000000000000000000000000 ]
    // Shared
    // [ 00XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX ]
    state: Arc<AtomicU64>,
}

impl Drop for RWLock {
    fn drop(&mut self) {
        if thread::panicking() {
            self.state.fetch_or(POISONED, Ordering::SeqCst);
        }
    }
}

impl Clone for RWLock {
    fn clone(&self) -> Self {
        RWLock {
            state: self.state.clone(),
        }
    }
}

impl RWLock {
    /// Construct a new lock.
    pub fn new() -> Self {
        RWLock {
            state: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Try to acquire shared access to the lock.
    /// Multiple `try_lock()` will succeed as long as the lock is unlocked,
    /// or locked in a shared state.
    /// Return `Ok(())` if the lock was acquired, else an error.
    /// `try_lock()` may fail if the lock is locked in exclusive state
    /// , if a thread is currently performing an operation on the lock,
    /// or if a thread owning a clone of this lock panicked.
    /// Call [`unlock()`](struct.RWLock.html#method.unlock) to
    /// unlock after succesfull `try_lock()`.
    pub fn try_lock(&self) -> Result<(), TryLockError<()>> {
        let count = self.state.load(Ordering::SeqCst);
        if (count & POISONED) != 0 {
            Err(TryLockError::Poisoned(()))
        } else if (count & EXCLUSIVE) != 0 {
            Err(TryLockError::WouldBlock(()))
        } else {
            match self.state.compare_exchange_weak(
                count,
                count + 1,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => Ok(()),
                Err(_) => Err(TryLockError::WouldBlock(())),
            }
        }
    }

    /// Try to acquire shared access to the lock.
    /// If lock is acquired, return input value wrapped around a
    /// [`RWLockGuard`](struct.RWLockGuard.html) such that when this
    /// value goes out of scope, the lock is released.
    /// If lock is not acquired, return the input value wrapped into
    /// an error.
    /// See [`try_lock()`](struct.RWLock.html#method.try_lock) for more
    /// details.
    pub fn try_lock_for<T>(
        &self,
        t: T,
    ) -> Result<RWLockGuard<T>, TryLockError<T>> {
        match self.try_lock() {
            Ok(_) => Ok(RWLockGuard::new(self, t)),
            Err(TryLockError::WouldBlock(_)) => {
                Err(TryLockError::WouldBlock(t))
            }
            Err(TryLockError::Poisoned(_)) => {
                Err(TryLockError::Poisoned(t))
            }
        }
    }

    /// Try to acquire exclusive access to the lock.
    /// `try_lock_mut()` will succeed only if the lock is unlocked.
    /// Return Ok(()) if the lock was acquired, else an error.
    /// `try_lock_mut()` may fail if the lock is already locked, if
    /// a thread is currently performing an operation on the lock or
    /// if a thread owning a clone of this lock panicked.
    /// Call [`unlock()`](struct.RWLock.html#method.unlock) to unlock
    /// after succesfull `try_lock_mut()`.
    pub fn try_lock_mut(&self) -> Result<(), TryLockError<()>> {
        let count = self.state.load(Ordering::SeqCst);
        if (count & POISONED) != 0 {
            Err(TryLockError::Poisoned(()))
        } else if (count & EXCLUSIVE) != 0 {
            Err(TryLockError::WouldBlock(()))
        } else {
            match self.state.compare_exchange_weak(
                0,
                EXCLUSIVE,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => Ok(()),
                Err(_) => Err(TryLockError::WouldBlock(())),
            }
        }
    }

    /// Try to acquire exlusive access to the lock.
    /// If lock is acquired, return input value wrapped around a
    /// [`RWLockGuard`](struct.RWLockGuard.html) such that when this
    /// value goes out of scope, the lock is released.
    /// If lock is not acquired, return the input value wrapped into
    /// an error.
    /// See [`try_lock_mut()`](struct.RWLock.html#method.try_lock) for more
    /// details.
    pub fn try_lock_mut_for<T>(
        &self,
        t: T,
    ) -> Result<RWLockGuard<T>, TryLockError<T>> {
        match self.try_lock_mut() {
            Ok(_) => Ok(RWLockGuard::new(self, t)),
            Err(TryLockError::Poisoned(_)) => {
                Err(TryLockError::Poisoned(t))
            }
            Err(TryLockError::WouldBlock(_)) => {
                Err(TryLockError::WouldBlock(t))
            }
        }
    }

    /// Hang until shared access to the lock is granted.
    /// Call [`unlock()`](struct.RWLock.html#method.unlock) to
    /// unlock after succesfull `lock()`.
    pub fn lock(&self) -> Result<(), LockError<()>> {
        let mut nanos: u64 = 1;
        loop {
            match self.try_lock() {
                Ok(_) => break Ok(()),
                Err(TryLockError::Poisoned(_)) => {
                    break Err(LockError::Poisoned(()))
                }
                Err(TryLockError::WouldBlock(_)) => {
                    nanos = max(nanos * 2, 1000);
                    thread::sleep(time::Duration::from_nanos(nanos));
                }
            }
        }
    }

    /// Hang until shared access to the lock is granted.
    /// When lock is acquired, input value is wrapped around a
    /// [`RWLockGuard`](struct.RWLockGuard.html) such that when it
    /// goes out of scope, the lock is released.
    /// `lock_for()` may fail if a thread is currently performing an
    /// operation on the lock or if a thread owning a clone of this
    /// lock panicked. In such a case, the input is returned wrapped
    /// inside an error.
    pub fn lock_for<T>(
        &self,
        t: T,
    ) -> Result<RWLockGuard<T>, LockError<T>> {
        match self.lock() {
            Ok(_) => Ok(RWLockGuard::new(self, t)),
            Err(LockError::Poisoned(_)) => Err(LockError::Poisoned(t)),
        }
    }

    /// Hang until exclusive access to the lock is granted.
    /// Call [`unlock()`](struct.RWLock.html#method.unlock) to
    /// unlock after succesfull `lock_mut()`.
    pub fn lock_mut(&self) -> Result<(), LockError<()>> {
        let mut nanos: u64 = 1;
        loop {
            match self.try_lock_mut() {
                Ok(()) => break Ok(()),
                Err(TryLockError::Poisoned(_)) => {
                    break Err(LockError::Poisoned(()))
                }
                Err(TryLockError::WouldBlock(_)) => {
                    nanos = max(nanos * 2, 1000);
                    thread::sleep(time::Duration::from_nanos(nanos));
                }
            }
        }
    }

    /// Hang until exclusive access to the lock is granted.
    /// When lock is acquired, input value is wrapped around a
    /// [`RWLockGuard`](struct.RWLockGuard.html) such that when it
    /// goes out of scope, the lock is released.
    /// `lock_for()` may fail if a thread is currently performing an
    /// operation on the lock or if a thread owning a clone of this
    /// lock panicked. In such a case, the input is returned wrapped
    /// inside an error.
    pub fn lock_mut_for<T>(
        &self,
        t: T,
    ) -> Result<RWLockGuard<T>, LockError<T>> {
        match self.lock_mut() {
            Ok(_) => Ok(RWLockGuard::new(self, t)),
            Err(LockError::Poisoned(_)) => Err(LockError::Poisoned(t)),
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

/// A RWLock guard that unlock a lock when going out of scope.
/// Element inside a RWLockGuard can be accessed with Deref and DerefMut methods.
///
/// # Examples
///
/// ```ignore
/// use cache::lock::{RWLock, RWLockGuard};
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

impl<'a, T, V: Iterator<Item = T>> Iterator for RWLockGuard<'a, V> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.value.next()
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

//------------------------------------------------------------------------------------//
//                                        Tests                                       //
//------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::{LockError, RWLock, TryLockError};
    use std::thread;
    use std::thread::JoinHandle;

    #[test]
    fn test_lock() {
        let lock = RWLock::new();
        let num_threads = 1024;
        let mut threads: Vec<JoinHandle<_>> =
            Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let local_lock = lock.clone();
            threads.push(thread::spawn(move || {
                // No thread should be panicking.
                local_lock.lock().unwrap();
                // The thread cannot reacquire lock.
                match local_lock.try_lock_mut() {
                    Err(TryLockError::WouldBlock(_)) => {}
                    _ => panic!(
                        "Try should cannot reacquire lock or be poisoned."
                    ),
                }
            }));
        }

        while let Some(t) = threads.pop() {
            t.join().unwrap();
        }

        for _ in 0..num_threads {
            let local_lock = lock.clone();
            threads.push(thread::spawn(move || {
                local_lock.unlock();
            }));
        }

        while let Some(t) = threads.pop() {
            t.join().unwrap();
        }

        match lock.try_lock_mut() {
            Ok(_) => {}
            Err(_) => panic!("Should be able to lock unlocked lock."),
        }
    }

    #[test]
    fn test_poison() {
        let lock = RWLock::new();
        let poisoned_lock = lock.clone();
        assert!(thread::spawn(move || {
            poisoned_lock.lock().unwrap();
            panic!("Intended panic to poison the lock.")
        })
        .join()
        .is_err());

        match lock.try_lock() {
            Err(TryLockError::Poisoned(_)) => {} // Ok
            _ => panic!("Lock should be poisoned."),
        }

        match lock.lock() {
            Err(LockError::Poisoned(_)) => {} // Ok
            _ => panic!("Lock should be poisoned."),
        }
    }
}

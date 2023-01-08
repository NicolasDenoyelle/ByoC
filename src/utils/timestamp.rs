//! Fixed point in time used by time sensitive cache policies.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// timestamp trait representing a fixed point in time.
///
/// ## Traits:
///
/// * `Ord`: Timestamp must be orderable such that it is possible to
/// to witness progressing time.
/// * `Clone`: Since timestamps are supposed to be lightweight it is reasonable
/// to ask for it to be copy in order to facilitate management of object using
/// timestamps.
pub trait Timestamp: Ord + Eq + PartialOrd + PartialEq + Copy {
    /// A point in time representing the present.
    fn now() -> Self;
    /// The difference between two points in time.
    fn diff(&self, other: &Self) -> f32;
}

//----------------------------------------------------------------------------//
// Counter based timestamp.                                                   //
//----------------------------------------------------------------------------//

/// Global counter for [`Counter`].
static mut COUNTER_STATE: AtomicU64 = AtomicU64::new(0);

/// A timestamp based on a global atomic counter.
///
/// Everytime this [`Timestamp`] is used to read the current time
/// the global atomic counter is incremented.
///
/// ```
/// use byoc::utils::timestamp::{Timestamp, Counter};
///
/// // Counters are strictly ascending
/// assert_eq!(u64::from(Counter::now()) + 1, u64::from(Counter::now()));
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Counter {
    value: u64,
}

impl From<Counter> for u64 {
    fn from(c: Counter) -> u64 {
        c.value
    }
}

impl Timestamp for Counter {
    /// [`Timestamp`](trait.Timestamp.html) creation.
    ///
    /// This will perform a `fetch_add()` atomic operation on a global counter
    /// and return the initial value of the counter in a [`Counter`].
    fn now() -> Self {
        // This may overflow.
        let v = unsafe { COUNTER_STATE.fetch_add(1u64, Ordering::SeqCst) };
        Counter { value: v }
    }

    /// Compute the difference then cast the result to f32.
    ///
    /// ```
    /// use byoc::utils::timestamp::{Timestamp, Counter};
    ///
    /// let t0 = Counter::now();
    /// assert_eq!(Counter::now().diff(&t0), 1.0);
    /// ```
    fn diff(&self, other: &Self) -> f32 {
        (self.value - other.value) as f32
    }
}

//----------------------------------------------------------------------------//
// Clock based timestamp.                                                     //
//----------------------------------------------------------------------------//

/// A timestamp based on a monotonic clock.
///
/// This timestamp is wrapper around [`std::time::Instant`].
/// The granularity of the clock is the nanosecond.
///
/// ```
/// use byoc::utils::timestamp::{Timestamp, Clock};
///    
/// assert!(Clock::now() < Clock::now());
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Clock {
    t: Instant,
}

impl Timestamp for Clock {
    /// Create a timestamp of current time with a nanoseconds granularity.
    fn now() -> Self {
        Clock { t: Instant::now() }
    }

    fn diff(&self, other: &Self) -> f32 {
        self.t.duration_since(other.t).as_nanos() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, Counter, Timestamp};

    fn test_timestamp<T: Timestamp + std::fmt::Debug>() {
        let t0 = T::now();
        let mut ti = T::now();

        for _ in 0..10 {
            let tj = T::now();
            assert!(tj >= ti);
            assert!(tj >= t0);
            // assert!(tj == tj);
            ti = tj;
        }
    }

    #[test]
    fn test_clock() {
        test_timestamp::<Clock>();
    }

    #[test]
    fn test_counter() {
        test_timestamp::<Counter>();
    }
}

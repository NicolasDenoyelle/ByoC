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
pub trait Timestamp: Ord + Clone {
    /// Timestamp default constructor.
    fn new() -> Self;
    /// Timestamp difference to f32.
    fn diff(&self, other: &Self) -> f32;
}

//----------------------------------------------------------------------------//
// Counter based timestamp.                                                   //
//----------------------------------------------------------------------------//

/// Global counter for [`Counter`](struct.Counter.html) implementation
/// of [`Timestamp`](trait.Timestamp.html).
///
/// It is modified in a none thread safe way.
/// However, it is ok to assume that threads issuing a timestamp
/// at the same moment will get similar timestamp (more or less).
/// Therefore, it is not an issue if `COUNTER_STATE` is incremented concurrently.
/// This is a very big counter. Unlikely to overflow.
static mut COUNTER_STATE: AtomicU64 = AtomicU64::new(0);

/// A timestamp based on a global counter.
///
/// ```
/// use cache::utils::timestamp::{Timestamp, Counter};
///
/// // Counters are strictly ascending
/// assert!(Counter::new() < Counter::new());
/// ```
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct Counter {
    value: u64,
}

impl Timestamp for Counter {
    /// [`Timestamp`](trait.Timestamp.html) creation.
    ///
    /// This will copy the global counter before incrementing it.
    /// The copy before increment is returned.
    fn new() -> Counter {
        let v: u64;
        // SAFETY: This thread safe because we are doing an atomic
        // operation on the global variable. This may overflow.
        unsafe { v = COUNTER_STATE.fetch_add(1u64, Ordering::SeqCst) }
        Counter { value: v }
    }

    /// Convert to f32 then compute difference.
    ///
    /// ```
    /// use cache::utils::timestamp::{Timestamp, Counter};
    ///
    /// let t0 = Counter::new();
    /// let t1 = Counter::new();
    /// assert!(t1.diff(&t0) == 1.0);
    /// ```
    fn diff(&self, other: &Self) -> f32 {
        (self.value - other.value) as f32
    }
}

//----------------------------------------------------------------------------//
// Clock based timestamp.                                                     //
//----------------------------------------------------------------------------//

/// A timestamp based on monotonic clock.
///
/// ## Details
///
/// This is implemented using rust `Instant` and `Duration`.
/// To this date the granularity of the clock is the nanosecond.
///
/// ```
/// use cache::utils::timestamp::{Timestamp, Clock};
///    
/// assert!(Clock::new() < Clock::new());
/// ```
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Clock {
    t: Instant,
}

impl Timestamp for Clock {
    /// Taking a timestamp on current time.
    ///
    /// The clock granularity is nanoseconds.
    fn new() -> Clock {
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
        let t0 = T::new();
        let mut ti = T::new();

        for _ in 0..10 {
            let tj = T::new();
            assert!(tj >= ti);
            assert!(tj >= t0);
            assert!(tj == tj);
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

use crate::policies::timestamp::Timestamp;
use crate::policies::{Reference, ReferenceFactory};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::cmp::{Ord, Ordering};

//------------------------------------------------------------------------//
// Least Frequently Used Policy on cache references                       //
//------------------------------------------------------------------------//

/// This structure keeps track of access statistics.
/// The access statistics is the exponential average of
/// time difference between accesses.
/// The exponential average can grant either higher or lower weight to
/// recent touches than older ones while time differences grants higher
/// weight to unfrequent touches, thus increasing the likelihood of
/// eviction for unfrequently touched element even if they have been
/// frequently touched long ago, and decreasing the likelihood of eviction
/// for recently inserted elements.
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stats<T: Timestamp + Copy> {
    /// Last touch timestamp
    last: T,
    /// Exponential average of time differences between access.
    eavg: f32,
    /// Exponent to use when incrementing exponential average.
    exponent: f32,
}

impl<T: Timestamp + Copy> Stats<T> {
    /// Create a new statistic tracker with a set exponent.
    /// On each touch, the weight of previous accesses is divided
    /// by this exponent.
    pub fn new(exponent: f32) -> Self {
        Stats {
            exponent,
            last: T::new(),
            eavg: 0f32,
        }
    }

    /// Update statistic tracker with a new access.
    /// This function gets the current time timestamp,
    /// computes the time difference between now and the timestamp of
    /// last touch. The difference is summed to the current statistic
    /// and the total statistic is then divided by the tracker exponent.
    pub fn touch(&mut self) {
        let last = T::new();
        let diff = last.diff(&self.last);
        self.last = last;
        self.eavg = diff + self.eavg / self.exponent;
    }

    /// Read the current statistic of this tracker.
    pub fn score(&self) -> f32 {
        T::new().diff(&self.last) + self.eavg / self.exponent
    }
}

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Frequently Used (LRFU) eviction policy.
///
/// `LRFUCell` references implement an order
/// based on the Least Recently Frequently Used (LRFU) policy.
///
/// See [`LRFU`](struct.LRFU.html)
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LRFUCell<V, T: Timestamp + Copy> {
    /// Reference value.
    value: V,
    stats: Cell<Stats<T>>,
}

/// Value wrappers implementing Least Recently Frequently Used ordering.
///
/// `LRFU` wraps values into cells implementing LRFU ordering policy.
/// It tries to keep in cache frequently used elements while giving a chance
/// to recently added but not frequently used elements to stay in the cache.
/// When a cache lookup occurs the state of the cell is updated
/// according to the number of times it is accessed and the timestamp of
/// accesses.
///
/// When a cache element wrapped into a LRFU cell is accessed, its
/// statistic is updated as follow:
/// the time difference between now and the timestamp of
/// last touch is computed. The difference is summed to the current
/// statistic and the total statistic is then divided by the policy
/// `exponent`.
///
/// See [`LRFU::new()`](struct.LRFU.html#tymethod.new)
///
/// ## Examples
///
/// ```
/// use byoc::{Array, Policy};
/// use byoc::policies::LRFU;
///
/// // let c = Policy::new(Array::new(3), LRFU::new(2.0));
/// ```
pub struct LRFU<T: Timestamp + Copy> {
    exponent: f32,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Timestamp + Copy> Clone for LRFU<T> {
    fn clone(&self) -> Self {
        LRFU {
            exponent: self.exponent,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Timestamp + Copy> LRFU<T> {
    /// Construct a LRFU references factory.
    ///
    /// The `exponent` decay must be strictly greater than 0.
    /// The greater the exponent (>>1) the closer to Least Recently Used
    /// this policy becomes.
    /// The smaller (>=1) the exponent, the closer to Least Frequently Used
    /// this  policy gets.
    /// If exponent is < 1, then the policy put more weight on old elements.
    ///
    /// See [`LRFU`](struct.LRFU.html)
    pub fn new(exponent: f32) -> Self {
        LRFU {
            exponent,
            phantom: std::marker::PhantomData,
        }
    }
}

unsafe impl<T: Timestamp + Copy> Send for LRFU<T> {}
unsafe impl<T: Timestamp + Copy> Sync for LRFU<T> {}

impl<V, T: Timestamp + Copy> ReferenceFactory<V, LRFUCell<V, T>>
    for LRFU<T>
{
    fn wrap(&mut self, v: V) -> LRFUCell<V, T> {
        LRFUCell::new(v, self.exponent)
    }
}

impl<V, T: Timestamp + Copy> LRFUCell<V, T> {
    /// Construct a [`LRFUCell`](struct.LRFUCell.html) cache reference.
    ///
    /// See [`LRFU`](struct.LRFU.html) and
    /// [`LRFU::new()`](struct.LRFU.html#tymethod.new)
    /// for more details on exponent argument.
    pub fn new(v: V, exponent: f32) -> Self {
        if exponent <= 0.0 {
            panic!("LRFUCell exponent cannot be <= 0.");
        }
        LRFUCell {
            value: v,
            stats: Cell::new(Stats::new(exponent)),
        }
    }

    pub fn touch(&self) {
        // SAFETY:
        // self.stats is initialized cell.
        unsafe {
            (*self.stats.as_ptr()).touch();
        }
    }
}

impl<V, T: Timestamp + Copy> Ord for LRFUCell<V, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // SAFETY:
        // self.stats and other.stats are initialized.
        unsafe {
            (*self.stats.as_ptr())
                .score()
                .partial_cmp(&(*other.stats.as_ptr()).score())
                .unwrap_or(Ordering::Equal)
        }
    }
}

impl<V, T: Timestamp + Copy> PartialOrd for LRFUCell<V, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, T: Timestamp + Copy> PartialEq for LRFUCell<V, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<V, T: Timestamp + Copy> Eq for LRFUCell<V, T> {}

impl<V, T: Timestamp + Copy> Reference<V> for LRFUCell<V, T> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get(&self) -> &V {
        self.touch();
        &self.value
    }
    fn get_mut(&mut self) -> &mut V {
        self.touch();
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::LRFUCell;
    use crate::policies::timestamp::Counter;
    use crate::policies::Reference;

    #[test]
    fn test_lrfu_ref() {
        let r0 = LRFUCell::<u32, Counter>::new(999, 2.0);
        let r1 = LRFUCell::<u32, Counter>::new(666, 2.0);
        r0.get();
        assert!(r0 < r1); // r0 is the most frequently and recently touched.
        r1.get();
        assert!(r1 < r0); // r0 and r1 are as frequently used but r1 is more
                          // recent.
        r0.get();
        assert!(r0 < r1); // r0 is the most frequently and recently touched.
        r0.get();
        r1.get();
        assert!(r0 < r1); // r0 is more frequently and only slightly older than
                          // r1.
    }
}

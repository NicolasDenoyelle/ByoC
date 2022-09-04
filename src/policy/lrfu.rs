use crate::policy::timestamp::Timestamp;
use crate::policy::{Reference, ReferenceFactory};
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
            last: T::now(),
            eavg: 0f32,
        }
    }

    /// Update statistic tracker with a new access.
    /// This function gets the current time timestamp,
    /// computes the time difference between now and the timestamp of
    /// last touch. The difference is summed to the current statistic
    /// and the total statistic is then divided by the tracker exponent.
    pub fn touch(&mut self) {
        let last = T::now();
        let diff = last.diff(&self.last);
        self.last = last;
        self.eavg = diff + self.eavg / self.exponent;
    }

    /// Read the current statistic of this tracker.
    pub fn score(&self) -> f32 {
        T::now().diff(&self.last) + self.eavg / self.exponent
    }
}

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Frequently Used (Lrfu) eviction policy.
///
/// `LrfuCell` references implement an order
/// based on the Least Recently Frequently Used (Lrfu) policy.
///
/// See [`Lrfu`](struct.Lrfu.html)
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LrfuCell<V, T: Timestamp + Copy> {
    /// Reference value.
    value: V,
    stats: Cell<Stats<T>>,
}

/// Reference implementation of Least Recently Frequently Used ordering.
///
/// `Lrfu` wraps values into cells implementing Lrfu ordering policy.
/// It tries to keep in cache frequently used elements while giving a chance
/// to recently added but not frequently used elements to stay in the cache.
/// When a cache lookup occurs the state of the cell is updated
/// according to the number of times it is accessed and the timestamp of
/// accesses.
///
/// When a cache element wrapped into a Lrfu cell is accessed, its
/// statistic is updated as follow:
/// the time difference between now and the timestamp of
/// last touch is computed. The difference is summed to the current
/// statistic and the total statistic is then divided by the policy
/// `exponent`.
///
/// See [`Lrfu::new()`](struct.Lrfu.html#tymethod.new)
///
/// ## Examples
///
/// ```
/// use byoc::{Array, Policy};
/// use byoc::policy::Lrfu;
///
/// // let c = Policy::new(Array::new(3), Lrfu::new(2.0));
/// ```
pub struct Lrfu<T: Timestamp + Copy> {
    exponent: f32,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Timestamp + Copy> Clone for Lrfu<T> {
    fn clone(&self) -> Self {
        Lrfu {
            exponent: self.exponent,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Timestamp + Copy> Lrfu<T> {
    /// Construct a Lrfu references factory.
    ///
    /// The `exponent` decay must be strictly greater than 0.
    /// The greater the exponent (>>1) the closer to Least Recently Used
    /// this policy becomes.
    /// The smaller (>=1) the exponent, the closer to Least Frequently Used
    /// this  policy gets.
    /// If exponent is < 1, then the policy put more weight on old elements.
    ///
    /// See [`Lrfu`](struct.Lrfu.html)
    pub fn new(exponent: f32) -> Self {
        Lrfu {
            exponent,
            phantom: std::marker::PhantomData,
        }
    }
}

unsafe impl<T: Timestamp + Copy> Send for Lrfu<T> {}
unsafe impl<T: Timestamp + Copy> Sync for Lrfu<T> {}

impl<V, T: Timestamp + Copy> ReferenceFactory<V> for Lrfu<T> {
    type Item = LrfuCell<V, T>;
    fn wrap(&mut self, v: V) -> Self::Item {
        LrfuCell::new(v, self.exponent)
    }
}

impl<V, T: Timestamp + Copy> LrfuCell<V, T> {
    /// Construct a [`LrfuCell`](struct.LrfuCell.html) cache reference.
    ///
    /// See [`Lrfu`](struct.Lrfu.html) and
    /// [`Lrfu::new()`](struct.Lrfu.html#tymethod.new)
    /// for more details on exponent argument.
    pub fn new(v: V, exponent: f32) -> Self {
        if exponent <= 0.0 {
            panic!("LrfuCell exponent cannot be <= 0.");
        }
        LrfuCell {
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

impl<V, T: Timestamp + Copy> Ord for LrfuCell<V, T> {
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

impl<V, T: Timestamp + Copy> PartialOrd for LrfuCell<V, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, T: Timestamp + Copy> PartialEq for LrfuCell<V, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<V, T: Timestamp + Copy> Eq for LrfuCell<V, T> {}

impl<V, T: Timestamp + Copy> Reference<V> for LrfuCell<V, T> {
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
    use super::LrfuCell;
    use crate::policy::timestamp::Counter;
    use crate::policy::Reference;

    #[test]
    fn test_lrfu_ref() {
        let r0 = LrfuCell::<u32, Counter>::new(999, 2.0);
        let r1 = LrfuCell::<u32, Counter>::new(666, 2.0);
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

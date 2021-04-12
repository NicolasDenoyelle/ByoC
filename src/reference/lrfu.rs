use crate::reference::Reference;
use crate::timestamp::Timestamp;
use std::cell::Cell;
use std::cmp::{Ord, Ordering};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Least Frequently Used Policy on cache references                       //
//------------------------------------------------------------------------//

#[derive(Clone, Debug, Copy)]
pub struct Stats<T: Timestamp + Copy> {
    /// Last touch timestamp
    last: T,
    /// Exponential average of time differences between access.
    eavg: f32,
    /// Exponent to use when incrementing exponential average.
    exponent: f32,
}

impl<T: Timestamp + Copy> Stats<T> {
    pub fn new(exponent: f32) -> Self {
        Stats {
            exponent: exponent,
            last: T::new(),
            eavg: 0f32,
        }
    }

    pub fn touch(&mut self) {
        let last = T::new();
        let diff = last.diff(&self.last);
        self.last = last;
        self.eavg = diff + self.eavg / self.exponent;
    }

    pub fn score(&self) -> f32 {
        T::new().diff(&self.last) + self.eavg / self.exponent
    }
}

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Frequently Used (LRFU) eviction policy.
///
/// ## Details
///
/// `LRFU` references implement an order
/// based on the Least Recently Frequently Used (LRFU) policy.
/// It tries to keep in cache frequently used elements while giving a chance
/// to recently added but not frequently usef elements to stay in the cache.
/// When a cache lookup occures the state of the reference is updated according
/// to the number of times it is accessed and the timestamp of accesses.
///
/// ## Generics
///
/// * `V`: type of value held in reference.
/// * `T`: a type implementing [`Timestamp`](../timestamp/trait.Timestamp.html) trait
/// for measuring access frequency.
///
/// ## Examples
///
/// ```
/// use cache::reference::{Reference, LRFU};
/// use cache::timestamp::{Timestamp, Clock};
///
/// // Least Recently Used cache reference storing f32 values and
/// // counting time with Counter.
/// let mut r0 = LRFU::<u32, Clock>::new(999, 2.0);
/// let mut r1 = LRFU::<u32, Clock>::new(666, 2.0);
/// *r0;
/// assert!( r0 < r1 ); // r0 is the most frequently and recently touched.
/// *r1;
/// assert!( r1 < r0 ); // r0 and r1 are as frequently used but r1 is more recent.
/// *r0;
/// assert!( r0 < r1 ); // r0 is the most frequently and recently touched.
/// *r0;
/// *r1;
/// assert!( r0 < r1 ); // r0 is more frequently and slightly older than r1.
/// ```
#[derive(Debug)]
pub struct LRFU<V, T: Timestamp + Copy> {
    /// Reference value.
    value: V,
    stats: Cell<Stats<T>>,
}

impl<V, T: Timestamp + Copy> LRFU<V, T> {
    /// Construct a [`LRFU`](struct.LRFU.html) cache reference.
    ///
    /// The importance of "recently" and "frequently" used can be
    /// weighted for the computation of ordering of references in cache.
    /// This computation is done in `score()` of
    /// [`LRFU`](struct.LRFU.html) references.
    ///
    /// ## Arguments:    
    ///
    /// * `v`: The value to store in the cache reference.
    /// * `exponent`: The exponential decay of weight of old access. Must be > 0.
    /// The greater the exponent the closer to LRU policy gets.
    /// The smaller (>=1) the exponent, the closer to LFU policy gets.
    /// If exponent is < 1, then the policy put more weight on old elements.
    pub fn new(v: V, exponent: f32) -> Self {
        if exponent <= 0.0 {
            panic!("LRFU exponent cannot be <= 0.");
        }
        LRFU {
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

impl<V, T: Timestamp + Copy> Deref for LRFU<V, T> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.touch();
        &self.value
    }
}

impl<V, T: Timestamp + Copy> DerefMut for LRFU<V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.touch();
        &mut self.value
    }
}

impl<V, T: Timestamp + Copy> Ord for LRFU<V, T> {
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

impl<V, T: Timestamp + Copy> PartialOrd for LRFU<V, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<V, T: Timestamp + Copy> PartialEq for LRFU<V, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}

impl<V, T: Timestamp + Copy> Eq for LRFU<V, T> {}

impl<V, T: Timestamp + Copy> Reference<V> for LRFU<V, T> {}

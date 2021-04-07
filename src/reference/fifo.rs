use crate::reference::Reference;
use crate::timestamp::{Counter, Timestamp};
use std::cmp::{Ord, Ordering};
use std::ops::{Deref, DerefMut};

//----------------------------------------------------------------------------//
// FIFO eviction policy                                                       //
//----------------------------------------------------------------------------//

/// Implementation of [`Reference`](trait.Reference.html) with
/// a First In First Out eviction policy.
///
/// ## Details
///
/// `FIFO` cache references are not affected by `touch()` call.
/// Eviction of these references is not affected by cache lookups.
///
/// ## Generics:
///
/// * `V`: The type of the value held in the
/// [`Reference`](trait.Reference.html).
///
/// ## Examples
///
/// ```
/// use cache::reference::{Reference, FIFO};
///
/// let p0 = FIFO::new("item0");
/// let p1 = FIFO::new("item1");
/// assert!( p0 > p1 );
/// assert!( p1 < p0 );
/// assert!( p0 == p0 );
/// assert!( p1 == p1 );
/// ```
#[derive(Debug)]
pub struct FIFO<V> {
    value: V,
    timestamp: Counter,
}

impl<V> FIFO<V> {
    pub fn new(v: V) -> Self {
        FIFO {
            value: v,
            timestamp: Counter::new(),
        }
    }
}

impl<V> Reference<V> for FIFO<V> {
    fn unwrap(self) -> V {
        self.value
    }
    fn clone(&self, value: V) -> Self {
        FIFO {
            value: value,
            timestamp: self.timestamp,
        }
    }
}

impl<V> Deref for FIFO<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for FIFO<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<V> Ord for FIFO<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp > other.timestamp {
            Ordering::Less
        } else if self.timestamp < other.timestamp {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<V> PartialOrd for FIFO<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> PartialEq for FIFO<V> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<V> Eq for FIFO<V> {}

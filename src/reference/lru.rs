use crate::reference::Reference;
use crate::timestamp::{Counter, Timestamp};
use std::cell::Cell;
use std::cmp::{Ord, Ordering};
use std::ops::{Deref, DerefMut};

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Used (LRU) eviction policy.
///
/// ## Details
///
/// `LRU` references implement an order based on the Least Recently Used (LRU) policy.
/// It tries to keep in cache elements that were recently accessed.
///
/// ## Generics
///
/// * `V`: type of value held in reference.
/// ## Examples
///
/// ```
/// use cache::reference::{Reference, LRU};
///
/// let mut lfu_0 = LRU::<u32>::new(999);
/// let mut lfu_1 = LRU::<u32>::new(666);
/// assert!( lfu_0 > lfu_1 ); // lfu_1 is the most recently created.
/// *lfu_0;
/// assert!( lfu_0 < lfu_1 ); // lfu_0 is the most recently used.
/// ```

#[derive(Debug)]
pub struct LRU<V> {
    value: V,
    /// Last access time.
    timestamp: Cell<Counter>,
}

impl<V> LRU<V> {
    pub fn new(e: V) -> Self {
        LRU {
            value: e,
            timestamp: Cell::new(Counter::new()),
        }
    }
}

impl<V> Deref for LRU<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.timestamp.set(Counter::new());
        &self.value
    }
}

impl<V> DerefMut for LRU<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.timestamp.set(Counter::new());
        &mut self.value
    }
}

impl<V> Ord for LRU<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<V> PartialOrd for LRU<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> PartialEq for LRU<V> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<V> Eq for LRU<V> {}

impl<V> Reference<V> for LRU<V> {}

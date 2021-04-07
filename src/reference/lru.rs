use crate::reference::Reference;
use crate::timestamp::{Counter, Timestamp};
use std::cmp::{Ord, Ordering};
use std::ops::{Deref, DerefMut};

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Used (LRU) eviction policy.
///
/// ## Details
///
/// `LRU` references implement an order based on the Least Recently Used (LRU) policy.
/// It tries to keep in cache elements that were recently touched.
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
/// lfu_0.touch();
/// assert!( lfu_0 < lfu_1 ); // lfu_0 is the most recently used.
/// ```

#[derive(Debug)]
pub struct LRU<V> {
    value: V,
    /// Last `touch()` time.
    timestamp: Counter,
}

impl<V> LRU<V> {
    pub fn new(e: V) -> Self {
        LRU {
            value: e,
            timestamp: Counter::new(),
        }
    }
}

impl<V> Reference<V> for LRU<V> {
    fn unwrap(self) -> V {
        self.value
    }
    fn touch(&mut self) -> &mut Self {
        self.timestamp = Counter::new();
        self
    }
    fn clone(&self, value: V) -> Self {
        LRU {
            value: value,
            timestamp: self.timestamp.clone(),
        }
    }
    fn replace(&mut self, value: V) -> V {
        std::mem::replace(&mut self.value, value)
    }
}

impl<V> Deref for LRU<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for LRU<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
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

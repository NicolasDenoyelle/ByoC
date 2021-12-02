use crate::policy::timestamp::Timestamp;
use crate::policy::{Reference, ReferenceFactory};
use std::cell::Cell;
use std::cmp::{Ord, Ordering};

/// Implementation of [`Reference`](trait.Reference.html)
/// with a Least Recently Used (LRU) eviction policy.
///
/// See /// See [`LRU`](struct.LRU.html)
#[derive(Debug)]
pub struct LRUCell<V, T: Timestamp> {
    value: V,
    /// Last access time.
    timestamp: Cell<T>,
}

/// Value wrappers implementing Least Recently Used ordering.
///
/// `LRU` wraps values into cells implementing LRU ordering policy.
/// The purpose of this policy is to keep in the cache the most recently
/// used elements while the least recently used one are elected for
/// eviction.
///
/// LRU implementation keep a timestamp of last access in the cell wrapping
/// the value on which to track last access. When the value is accessed
/// the timestamp is updated to the time of the access.
///
/// ## Examples
///
/// ```
/// use cache::container::Vector;
/// use cache::policy::{Policy, LRU};
/// use cache::policy::timestamp::Clock;
///
/// // let c = Policy::new(Vector::new(3), LRU::<Clock>::new());
/// ```
pub struct LRU<T: Timestamp> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: Timestamp> LRU<T> {
    pub fn new() -> Self {
        LRU {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<V, T: Timestamp> ReferenceFactory<V, LRUCell<V, T>> for LRU<T> {
    fn wrap(&mut self, v: V) -> LRUCell<V, T> {
        LRUCell {
            value: v,
            timestamp: Cell::new(T::new()),
        }
    }
}

unsafe impl<T: Timestamp> Send for LRU<T> {}
unsafe impl<T: Timestamp> Sync for LRU<T> {}

impl<V, T: Timestamp> LRUCell<V, T> {
    pub fn new(e: V) -> Self {
        LRUCell {
            value: e,
            timestamp: Cell::new(T::new()),
        }
    }
}

impl<V, T: Timestamp> Ord for LRUCell<V, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.get().cmp(&other.timestamp.get()) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<V, T: Timestamp> PartialOrd for LRUCell<V, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, T: Timestamp> PartialEq for LRUCell<V, T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.get() == other.timestamp.get()
    }
}

impl<V, T: Timestamp> Eq for LRUCell<V, T> {}

impl<V, T: Timestamp> Reference<V> for LRUCell<V, T> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get<'a>(&'a self) -> &'a V {
        self.timestamp.set(T::new());
        &self.value
    }
    fn get_mut<'a>(&'a mut self) -> &'a mut V {
        self.timestamp.set(T::new());
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::LRUCell;
    use crate::policy::timestamp::Counter;
    use crate::policy::Reference;

    #[test]
    fn test_lru_ref() {
        let lfu_0 = LRUCell::<u32, Counter>::new(999u32);
        let lfu_1 = LRUCell::<u32, Counter>::new(666u32);
        assert!(lfu_0 > lfu_1); // lfu_1 is the most recently created.
        lfu_0.get();
        assert!(lfu_0 < lfu_1); // lfu_0 is the most recently used.
    }
}

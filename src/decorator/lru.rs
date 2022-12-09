use crate::decorator::{Decoration, DecorationFactory};
use crate::utils::timestamp::Timestamp;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::cmp::{Ord, Ordering};

/// Implementation of [`Decoration`](trait.Decoration.html)
/// with a Least Recently Used (Lru) eviction policy.
///
/// See /// See [`Lru`](struct.Lru.html)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LruCell<V, T: Timestamp> {
    value: V,
    /// Last access time.
    timestamp: Cell<T>,
}

/// Decoration implementation of Least Recently Used ordering.
///
/// `Lru` wraps values into cells implementing Lru ordering policy.
/// The purpose of this policy is to keep in the cache the most recently
/// used elements while the least recently used one are elected for
/// eviction.
///
/// Lru implementation keep a timestamp of last access in the cell wrapping
/// the value on which to track last access. When the value is accessed
/// the timestamp is updated to the time of the access.
///
/// ## Examples
///
/// ```
/// use byoc::{Array, Decorator};
/// use byoc::decorator::Lru;
/// use byoc::utils::timestamp::Clock;
///
/// // let c = Decorator::new(Array::new(3), Lru::<Clock>::new());
/// ```
pub struct Lru<T: Timestamp> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: Timestamp> Clone for Lru<T> {
    fn clone(&self) -> Self {
        Lru {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Timestamp> Lru<T> {
    pub fn new() -> Self {
        Lru {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Timestamp> Default for Lru<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V, T: Timestamp> DecorationFactory<V> for Lru<T> {
    type Item = LruCell<V, T>;
    fn wrap(&mut self, v: V) -> Self::Item {
        LruCell {
            value: v,
            timestamp: Cell::new(T::now()),
        }
    }
}

unsafe impl<T: Timestamp> Send for Lru<T> {}
unsafe impl<T: Timestamp> Sync for Lru<T> {}

impl<V, T: Timestamp> LruCell<V, T> {
    pub fn new(e: V) -> Self {
        LruCell {
            value: e,
            timestamp: Cell::new(T::now()),
        }
    }
}

impl<V, T: Timestamp> Ord for LruCell<V, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.get().cmp(&other.timestamp.get()) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<V, T: Timestamp> PartialOrd for LruCell<V, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, T: Timestamp> PartialEq for LruCell<V, T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.get() == other.timestamp.get()
    }
}

impl<V, T: Timestamp> Eq for LruCell<V, T> {}

impl<V, T: Timestamp> Decoration<V> for LruCell<V, T> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get(&self) -> &V {
        self.timestamp.set(T::now());
        &self.value
    }
    fn get_mut(&mut self) -> &mut V {
        self.timestamp.set(T::now());
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::LruCell;
    use crate::decorator::Decoration;
    use crate::utils::timestamp::Counter;

    #[test]
    fn test_lru_ref() {
        let lfu_0 = LruCell::<u32, Counter>::new(999u32);
        let lfu_1 = LruCell::<u32, Counter>::new(666u32);
        assert!(lfu_0 > lfu_1); // lfu_1 is the most recently created.
        lfu_0.get();
        assert!(lfu_0 < lfu_1); // lfu_0 is the most recently used.
    }
}

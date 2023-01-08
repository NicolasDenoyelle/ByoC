use crate::decorator::{Decoration, DecorationFactory};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
//----------------------------------------------------------------------------//
// Fifo eviction policy                                                       //
//----------------------------------------------------------------------------//

/// Implementation of [`Decoration`](trait.Decoration.html) with
/// a First In First Out eviction policy.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FifoCell<V> {
    value: V,
    counter: u64,
}

/// Decoration implementation of First In First Out ordering.
///
/// `Fifo` wraps values into cells implementing Fifo ordering policy.
/// It tries to keep in cache last inserted elements while evicting older
/// insertions.
///
/// Fifo implementations keeps a counter of fifo cells creation and
/// attached the counter value to the value wrapped into a Fifo cell.
/// Fifo cells are further ordering in reverse order of their counter value
/// such that the oldest counter are the one evicted first.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Decorator};
/// use byoc::utils::decorator::Fifo;
///
/// let mut c = Decorator::new(Array::new(3), Fifo::new());
/// assert_eq!(c.push(vec![("item1",1u16), ("item2",2u16), ("item0",3u16)])
///             .len(), 0);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
pub struct Fifo {
    counter: AtomicU64,
}

impl Fifo {
    pub fn new() -> Self {
        Fifo {
            counter: AtomicU64::new(0),
        }
    }
}

impl Default for Fifo {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Fifo {
    fn clone(&self) -> Self {
        Fifo {
            counter: AtomicU64::new(self.counter.load(Relaxed)),
        }
    }
}

impl<V> DecorationFactory<V> for Fifo {
    type Item = FifoCell<V>;

    fn wrap(&mut self, v: V) -> Self::Item {
        FifoCell {
            value: v,
            counter: self.counter.fetch_add(1, Relaxed),
        }
    }
}

unsafe impl Send for Fifo {}
unsafe impl Sync for Fifo {}

impl<V> Ord for FifoCell<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.counter.cmp(&other.counter) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<V> PartialOrd for FifoCell<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> PartialEq for FifoCell<V> {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter
    }
}

impl<V> Eq for FifoCell<V> {}

impl<V> Decoration<V> for FifoCell<V> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get(&self) -> &V {
        &self.value
    }
    fn get_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::FifoCell;
    #[test]
    fn test_fifo_ref() {
        let p0 = FifoCell {
            value: "item0",
            counter: 0u64,
        };
        let p1 = FifoCell {
            value: "item1",
            counter: 1u64,
        };
        assert!(p0 > p1);
        assert!(p1 < p0);
        // assert!(p0 == p0);
        // assert!(p1 == p1);
    }
}

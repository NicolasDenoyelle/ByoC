use crate::policy::{Reference, ReferenceFactory};
use std::cmp::{Ord, Ordering};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
//----------------------------------------------------------------------------//
// FIFO eviction policy                                                       //
//----------------------------------------------------------------------------//

/// Implementation of [`Reference`](trait.Reference.html) with
/// a First In First Out eviction policy.
#[derive(Debug)]
pub struct FIFOCell<V> {
    value: V,
    counter: u64,
}

/// Value wrappers implementing First In First Out ordering.
///
/// `FIFO` wraps values into cells implementing FIFO ordering policy.
/// It tries to keep in cache last inserted elements while evicting older
/// insertions.
///
/// FIFO implementations keeps a counter of fifo cells creation and
/// attached the counter value to the value wrapped into a FIFO cell.
/// FIFO cells are further ordered in reverse order of their counter value
/// such that the oldest counter are the one evicted first.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
/// use cache::policy::{Policy, FIFO};
///
/// let mut c = Policy::new(Vector::new(3), FIFO::new());
/// c.push(vec![("item1",()), ("item2",()), ("item0",())]);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
pub struct FIFO {
    counter: AtomicU64,
}

impl FIFO {
    pub fn new() -> Self {
        FIFO {
            counter: AtomicU64::new(0),
        }
    }
}

impl<V> ReferenceFactory<V, FIFOCell<V>> for FIFO {
    fn wrap(&mut self, v: V) -> FIFOCell<V> {
        FIFOCell {
            value: v,
            counter: self.counter.fetch_add(1, Relaxed),
        }
    }
}

unsafe impl Send for FIFO {}
unsafe impl Sync for FIFO {}

impl<V> Ord for FIFOCell<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.counter > other.counter {
            Ordering::Less
        } else if self.counter < other.counter {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<V> PartialOrd for FIFOCell<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> PartialEq for FIFOCell<V> {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter
    }
}

impl<V> Eq for FIFOCell<V> {}

impl<V> Reference<V> for FIFOCell<V> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get<'a>(&'a self) -> &'a V {
        &self.value
    }
    fn get_mut<'a>(&'a mut self) -> &'a mut V {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::FIFOCell;

    #[test]
    fn test_fifo_ref() {
        let p0 = FIFOCell {
            value: "item0",
            counter: 0u64,
        };
        let p1 = FIFOCell {
            value: "item1",
            counter: 1u64,
        };
        assert!(p0 > p1);
        assert!(p1 < p0);
        assert!(p0 == p0);
        assert!(p1 == p1);
    }
}
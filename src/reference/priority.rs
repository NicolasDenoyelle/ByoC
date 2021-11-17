use crate::reference::Reference;
use std::cmp::{Ord, Ordering};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Priority based policy on cache references                              //
//------------------------------------------------------------------------//

/// Implementation of [`Reference`](trait.Reference.html) with
/// an eviction policy based on fixed priority.
///
/// Eviction of these references is not affected by cache lookups.
///
/// ## Examples
///
/// ```
/// use cache::reference::{Reference, Priority};
///
/// let p0 = Priority::new("item0",0);
/// let p1 = Priority::new("item1",1);
/// assert!( p1 > p0 );
/// ```
#[derive(Debug)]
pub struct Priority<V, P: Ord> {
    value: V,
    priority: P,
}

impl<V, P: Ord + Clone> Priority<V, P> {
    /// Constructor with attribute settings.
    ///
    /// * `v`: The value held in the cache reference.
    /// * `p`: The priority value of this cache reference.
    /// High priorities are more likely to be evicted.
    pub fn new(v: V, p: P) -> Self {
        Priority {
            value: v,
            priority: p,
        }
    }
}

impl<V, P: Ord + Clone> Deref for Priority<V, P> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V, P: Ord + Clone> DerefMut for Priority<V, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<V, P: Ord + Clone> Ord for Priority<V, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl<V, P: Ord + Clone> PartialOrd for Priority<V, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<V, P: Ord + Clone> PartialEq for Priority<V, P> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<V, P: Ord + Clone> Eq for Priority<V, P> {}

impl<V, P: Ord + Clone> Reference<V> for Priority<V, P> {}

#[cfg(test)]
mod tests {
    use super::Priority;

    #[test]
    fn test() {
        let p0 = Priority::new("whatever", 0);

        for i in 1..1000 {
            let pn = Priority::new("whatever", i);
            assert!(pn == pn);
            assert!(pn > p0);
            assert!(p0 < pn);
        }
    }
}

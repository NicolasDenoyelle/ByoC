use crate::container::{Container, Get};
use crate::marker::Packed;
use std::cmp::Eq;

//------------------------------------------------------------------------//
// Container Stack                                                        //
//------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) wrapper to build multi-level
/// cache.
///
/// Stack container implements a stack of 2 containers.
/// Insertions will be performed at the bottom of the stack.
/// Pops on insertions are propagated from the bottom to the top of the
/// stack.
///
/// Container lookups will look from the bottom to the top of the stack
/// for matches.
/// Whenever a match is found, the reference is taken out of the container,
/// unwrapped and reinserted at the bottom of the container stack.
///
/// `pop()` invocation will search from the top to the bottom of the stack
/// for an element to evict.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Get, Stack, Vector};
///
/// // Create cache
/// let mut l1 = Vector::new(1);
/// let mut l2 = Vector::new(1);
/// let mut cache = Stack::new(l1, l2);
/// assert_eq!(cache.capacity(), 2);
///
/// // Populate cache
/// assert!(cache.push("first", 0).is_none());
/// assert!(cache.push("second", 3).is_none());
/// let mut first = cache.get(&"first");
///
/// // Cache overflow. Victim is the Least Recently used, i.e "second".
/// let victim = cache.push("third", 2).unwrap();
/// assert_eq!(victim.0, "second");
/// ```
pub struct Stack<C1, C2> {
    l1: C1,
    l2: C2,
}

impl<C1, C2> Stack<C1, C2> {
    /// Construct a Stack Cache.
    ///
    /// The stack spans from bottom (first element) to top (last) element
    /// of the list of containers provided as input.
    ///
    /// * `containers`: The list of containers composing the stack.
    pub fn new(l1: C1, l2: C2) -> Stack<C1, C2> {
        Stack { l1: l1, l2: l2 }
    }
}

impl<'a, K, V, C1, C2> Container<'a, K, V> for Stack<C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a + Ord,
    C1: Container<'a, K, V>,
    C2: Container<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.l1.capacity() + self.l2.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.l1.flush().chain(self.l2.flush()))
    }

    fn contains(&self, key: &K) -> bool {
        if self.l1.contains(key) {
            true
        } else {
            self.l2.contains(key)
        }
    }

    fn count(&self) -> usize {
        self.l1.count() + self.l2.count()
    }

    fn clear(&mut self) {
        self.l1.clear();
        self.l2.clear();
    }

    fn take(
        &'a mut self,
        key: &'a K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.l1.take(key).chain(self.l2.take(key)))
    }

    fn pop(&mut self) -> Option<(K, V)> {
        match self.l2.pop() {
            None => self.l1.pop(),
            Some(x) => Some(x),
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        match self.l1.push(key.clone(), reference) {
            None => None,
            Some((k, v)) => self.l2.push(k, v),
        }
    }
}

impl<'a, K, V, C1, C2, T> Get<'a, K, V> for Stack<C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a + Ord,
    C1: 'a + Container<'a, K, V> + Get<'a, K, V, Item = T>,
    C2: 'a + Container<'a, K, V> + Packed<'a, K, V>,
    T: 'a,
{
    type Item = T;
    fn get(&'a mut self, key: &K) -> Option<Self::Item> {
        if self.l1.contains(key) {
            self.l1.get(key)
        } else if self.l2.contains(key) {
            let key = key.clone();
            let (k, v) = { self.l2.take(&key).next() }.unwrap();
            if let Some((k, v)) = self.l1.push(k, v) {
                assert!(self.l2.push(k, v).is_none());
            }
            self.l1.get(&key)
        } else {
            None
        }
    }
}

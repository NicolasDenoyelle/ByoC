use crate::container::{Container, Get};
use crate::marker::Packed;
use crate::utils::flush::VecFlushIterator;
use std::cmp::Eq;

//------------------------------------------------------------------------//
// Container Stack                                                        //
//------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) wrapper to build multi-level
/// cache.
///
/// Stack container implements a stack of 2 containers.
/// It is a non-inclusive container, i.e a key cannot be present in multiple
/// containers of the stack.
///
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
        Box::new(VecFlushIterator {
            it: vec![self.l1.flush(), self.l2.flush()],
        })
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

    fn take(&mut self, key: &K) -> Option<V> {
        match self.l1.take(key) {
            None => self.l2.take(key),
            Some(r) => Some(r),
        }
    }

    fn pop(&mut self) -> Option<(K, V)> {
        let x1 = self.l1.pop();
        let x2 = self.l2.pop();
        match (x1, x2) {
            (None, None) => None,
            (None, Some(x)) => Some(x),
            (Some(x), None) => Some(x),
            (Some((k1, v1)), Some((k2, v2))) => {
                if v1 <= v2 {
                    assert!(self.l1.push(k1, v1).is_none());
                    Some((k2, v2))
                } else {
                    assert!(self.l2.push(k2, v2).is_none());
                    Some((k1, v1))
                }
            }
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        match (self.l1.push(key.clone(), reference), self.l2.take(&key)) {
            (None, None) => None,
            (None, Some(r)) => Some((key, r)),
            (Some((k, v)), None) => {
                if k == key && self.l1.contains(&key) {
                    Some((k, v))
                } else {
                    self.l2.push(k, v)
                }
            }
            (Some((k, v)), Some(r)) => {
                assert!(self.l2.push(k, v).is_none());
                Some((key, r))
            }
        }
    }
}

impl<'a, K, V, C1, C2> Packed<'a, K, V> for Stack<C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a + Ord,
    C1: Container<'a, K, V> + Packed<'a, K, V>,
    C2: Container<'a, K, V> + Packed<'a, K, V>,
{
}

impl<'a, K, V, C1, C2, T> Get<'a, K, V> for Stack<C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a + Ord,
    C1: Container<'a, K, V> + Get<'a, K, V, Item = T>,
    C2: Container<'a, K, V>,
    T: 'a,
{
    type Item = T;
    fn get(&'a mut self, key: &K) -> Option<T> {
        // Start with first container
        if self.l1.contains(key) {
            // Found! Stop here.
            return self.l1.get(key);
        }

        // Not Found. Find in l2 and move to l1.
        match self.l2.take(key) {
            // Not Found. Stop here.
            None => None,
            // Found!
            Some(v2) => {
                // Make some room in l1
                match self.l1.pop() {
                    // We made room in l1. Push result to l2.
                    Some((k1, v1)) => {
                        assert!(self.l2.push(k1, v1).is_none());
                    }
                    // l1 was empty already.
                    None => (),
                }
                // Push found value into l1 inorder to invoke get.
                match self.l1.push(key.clone(), v2) {
                    // Worked, return get method result.
                    None => self.l1.get(key),
                    // l1 cannot store elements at all.
                    // We put element back in l2.
                    Some((k2, v2)) => {
                        assert!(self.l2.push(k2, v2).is_none());
                        None
                    }
                }
            }
        }
    }
}

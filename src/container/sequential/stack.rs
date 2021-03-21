use crate::container::{Container, Insert, Iter, IterMut, Packed, Sequential};
use crate::reference::{FromValue, Reference};
use std::cmp::Eq;
use std::marker::PhantomData;

//----------------------------------------------------------------------------//
// Constiner Stack                                                            //
//----------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) wrapper to build multi-level cache.
///
/// Stack container implements a stack of 2 containers.
/// It is a non-inclusive container, i.e a key cannot be present in multiple
/// containers of the stack.
///
/// Insertions will be performed at the bottom of the stack.
/// Pops on insertions are propagated from the bottom to the top of the stack.
///
/// Container lookups will look from the bottom to the top of the stack for matches.
/// Whenever a match is found, the reference is taken out of the container,
/// unwrapped and reinserted at the bottom of the container stack.
///
/// `pop()` invocation will search from the top to the bottom of the stack for
/// an element to evict.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Insert, Sequential};
/// use cache::container::sequential::{Stack, Vector, Map};
/// use cache::reference::Default;
///
/// // Create cache
/// let mut l1 = Vector::<_,Default<_>>::new(1);
/// let mut stack = Map::<_,Default<_>>::new(1);
/// let mut cache = Stack::new(l1, stack);
/// assert_eq!(cache.capacity(), 2);
///
/// // Populate cache
/// assert!(cache.insert("first", 0).is_none());
/// assert!(cache.insert("second", 3).is_none());
/// let mut first = cache.get_mut(&"first");
///
/// // Cache overflow. Victim is the Least Recently used, i.e "second".
/// let victim = cache.insert("third", 2).unwrap();
/// assert_eq!(victim.0, "second");
/// ```
pub struct Stack<K, V, C1, C2>
where
    K: Clone + Eq,
    V: Ord,
    C1: Container<K, V>,
    C2: Container<K, V>,
{
    l1: C1,
    l2: C2,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V, R, C1, C2> Insert<K, V, R> for Stack<K, R, C1, C2>
where
    K: Clone + Eq,
    R: Reference<V> + FromValue<V>,
    C1: Container<K, R>,
    C2: Container<K, R>,
{
}

impl<K, V, C1, C2> Stack<K, V, C1, C2>
where
    K: Clone + Eq,
    V: Ord,
    C1: Container<K, V>,
    C2: Container<K, V>,
{
    /// Construct a Stack Cache.
    ///
    /// The stack spans from bottom (first element) to top (last) element
    /// of the list of containers provided as input.
    ///
    /// * `containers`: The list of containers composing the stack.
    pub fn new(l1: C1, l2: C2) -> Stack<K, V, C1, C2> {
        Stack {
            l1: l1,
            l2: l2,
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }
}

impl<K, V, C1, C2> Container<K, V> for Stack<K, V, C1, C2>
where
    K: Clone + Eq,
    V: Ord,
    C1: Container<K, V>,
    C2: Container<K, V>,
{
    fn capacity(&self) -> usize {
        self.l1.capacity() + self.l2.capacity()
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        let mut v = self.l1.flush();
        v.append(&mut self.l2.flush());
        v
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
        match self.l2.pop() {
            None => self.l1.pop(),
            Some(r) => Some(r),
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

impl<K, V, C1, C2> Packed<K, V> for Stack<K, V, C1, C2>
where
    K: Clone + Eq,
    V: Ord,
    C1: Container<K, V> + Packed<K, V>,
    C2: Container<K, V> + Packed<K, V>,
{
}

impl<K, V, R, C1, C2> Sequential<K, V, R> for Stack<K, R, C1, C2>
where
    K: Clone + Eq,
    R: Reference<V>,
    C1: Container<K, R> + Sequential<K, V, R>,
    C2: Container<K, R> + Sequential<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        match self.get_mut(key) {
            None => None,
            Some(v) => Some(v),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.l1.contains(key) {
            self.l1.get_mut(key)
        } else {
            match self.l2.take(key) {
                None => None,
                Some(r) => match self.l1.push(key.clone(), r) {
                    None => self.l1.get_mut(key),
                    Some((k, v2)) => {
                        assert!(self.l2.push(k.clone(), v2).is_none());
                        if &k == key {
                            self.l2.get_mut(key)
                        } else {
                            self.l1.get_mut(key)
                        }
                    }
                },
            }
        }
    }
}

//----------------------------------------------------------------------------//
// iterator for associative cache                                             //
//----------------------------------------------------------------------------//

impl<'a, K, V, R, C1, I1, C2, I2> Iter<'a, K, V, R> for Stack<K, R, C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a,
    R: 'a + Reference<V>,
    C1: 'a + Container<K, R> + Iter<'a, K, V, R, Iterator = I1>,
    I1: 'a + Iterator<Item = (&'a K, &'a V)>,
    C2: Container<K, R> + Iter<'a, K, V, R, Iterator = I2>,
    I2: 'a + Iterator<Item = (&'a K, &'a V)>,
{
    type Iterator = std::iter::Chain<I1, I2>;

    fn iter(&'a mut self) -> Self::Iterator {
        self.l1.iter().chain(self.l2.iter())
    }
}

impl<'a, K, V, R, C1, I1, C2, I2> IterMut<'a, K, V, R> for Stack<K, R, C1, C2>
where
    K: 'a + Clone + Eq,
    V: 'a,
    R: 'a + Reference<V>,
    C1: 'a + Container<K, R> + IterMut<'a, K, V, R, Iterator = I1>,
    I1: 'a + Iterator<Item = (&'a K, &'a mut V)>,
    C2: Container<K, R> + IterMut<'a, K, V, R, Iterator = I2>,
    I2: 'a + Iterator<Item = (&'a K, &'a mut V)>,
{
    type Iterator = std::iter::Chain<I1, I2>;

    fn iter_mut(&'a mut self) -> Self::Iterator {
        self.l1.iter_mut().chain(self.l2.iter_mut())
    }
}

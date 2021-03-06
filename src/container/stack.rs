use crate::container::{Buffered, Container, Get};
use crate::marker::Packed;
use std::marker::PhantomData;

//------------------------------------------------------------------------//
// Container Stack                                                        //
//------------------------------------------------------------------------//

/// [`Container`](trait.Container.html) wrapper to build multi-level
/// cache.
///
/// Stack container implements a stack of 2 containers.
/// Insertions will be performed at the bottom of the stack.
/// Pops on insertions are propagated from the bottom to the top of the
/// stack.
///
/// Container lookups will look from the bottom to the top of the stack
/// for matches.
///
/// [`pop()`](trait.Container.html#tymethod.pop)
/// invocation will search from the top to the bottom of the stack
/// for an element to evict.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Stack, Vector};
///
/// // Create cache
/// let mut l1 = Vector::new(1);
/// let mut l2 = Vector::new(1);
/// let mut cache = Stack::new(l1, l2);
/// assert_eq!(cache.capacity(), 2);
///
/// // Populate cache
/// assert!(cache.push("first", 0).is_none());
/// // First layer is full. "first" get pushed to the second layer
/// // while "second" lives in the first one.
/// assert!(cache.push("second", 3).is_none());
///
/// // Cache overflow. Victim is the max element in second layer.
/// let victim = cache.push("third", 2).unwrap();
/// assert_eq!(victim.0, "first");
/// ```
pub struct Stack<'a, K: 'a, V: 'a, C1, C2>
where
    C1: Container<'a, K, V>,
    C2: Container<'a, K, V>,
{
    l1: C1,
    l2: C2,
    unused_k: PhantomData<&'a K>,
    unused_v: PhantomData<&'a V>,
}

impl<'a, K: 'a, V: 'a, C1, C2> Stack<'a, K, V, C1, C2>
where
    C1: Container<'a, K, V>,
    C2: Container<'a, K, V>,
{
    /// Construct a Stack Cache.
    ///
    /// The stack spans from bottom (first element) to top (last) element
    /// of the list of containers provided as input.
    ///
    /// * `containers`: The list of containers composing the stack.
    pub fn new(l1: C1, l2: C2) -> Self {
        Stack {
            l1: l1,
            l2: l2,
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a, C1, C2> Container<'a, K, V>
    for Stack<'a, K, V, C1, C2>
where
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

    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
        Box::new(self.l1.take(key).chain(self.l2.take(key)))
    }

    fn pop(&mut self) -> Option<(K, V)> {
        match self.l2.pop() {
            None => self.l1.pop(),
            Some(x) => Some(x),
        }
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        match self.l1.push(key, reference) {
            None => None,
            Some((k, v)) => self.l2.push(k, v),
        }
    }
}

impl<'a, K: 'a, V: 'a, C1, C2> Packed<'a, K, V> for Stack<'a, K, V, C1, C2>
where
    C1: Container<'a, K, V> + Packed<'a, K, V>,
    C2: Container<'a, K, V> + Packed<'a, K, V>,
{
}

impl<'a, K, V, C1, C2> Get<'a, K, V> for Stack<'a, K, V, C1, C2>
where
    K: 'a,
    V: 'a,
    C1: Container<'a, K, V> + Get<'a, K, V>,
    C2: Container<'a, K, V> + Get<'a, K, V>,
{
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b> {
        Box::new(self.l1.get(key).chain(self.l2.get(key)))
    }

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b> {
        Box::new(self.l1.get_mut(key).chain(self.l2.get_mut(key)))
    }
}

impl<'a, K, V, C1, C2> Buffered<'a, K, V> for Stack<'a, K, V, C1, C2>
where
    K: 'a,
    V: 'a,
    C1: Container<'a, K, V> + Buffered<'a, K, V>,
    C2: Container<'a, K, V> + Buffered<'a, K, V>,
{
    fn push_buffer(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        self.l2.push_buffer(self.l1.push_buffer(elements))
    }
}

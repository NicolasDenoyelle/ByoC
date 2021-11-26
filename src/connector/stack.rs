use crate::{BuildingBlock, Get};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// BuildingBlock Stack                                                        //
//------------------------------------------------------------------------//

/// [`BuildingBlock`](../trait.BuildingBlock.html) wrapper to build
/// multi-level cache.
///
/// Stack container implements a stack of 2 containers.
/// Insertions will be performed at the bottom of the stack.
/// Pops on insertions are propagated from the bottom to the top of the
/// stack.
///
/// BuildingBlock lookups will look from the bottom to the top of the stack
/// for matches.
///
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop)
/// invocation will search from the top to the bottom of the stack
/// for an element to evict.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::connector::Stack;
/// use cache::container::Vector;
///
/// // Create cache
/// let mut l1 = Vector::new(1);
/// let mut l2 = Vector::new(1);
/// let mut cache = Stack::new(l1, l2);
/// assert_eq!(cache.capacity(), 2);
///
/// // Populate cache
/// assert!(cache.push(vec![("first", 0)]).pop().is_none());
/// // First layer is full. "first" get pushed to the second layer
/// // while "second" lives in the first one.
/// assert!(cache.push(vec![("second", 3)]).pop().is_none());
///
/// // Cache overflow.
/// let victim = cache.push(vec![("third", 2)]).pop().unwrap();
/// assert_eq!(victim.0, "third");
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
    pub fn new(l1: C1, l2: C2) -> Self {
        Stack { l1: l1, l2: l2 }
    }
}

impl<'a, K: 'a, V: 'a, C1, C2> BuildingBlock<'a, K, V> for Stack<C1, C2>
where
    C1: BuildingBlock<'a, K, V>,
    C2: BuildingBlock<'a, K, V>,
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

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.l1.take(key) {
            Some(x) => Some(x),
            None => self.l2.take(key),
        }
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut v = self.l2.pop(n);

        if v.len() < n {
            v.append(&mut self.l1.pop(n - v.len()));
        }
        v
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        self.l2.push(self.l1.push(elements))
    }
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

enum DualCell<V, A, B>
where
    A: Deref<Target = V>,
    B: Deref<Target = V>,
{
    Atype(A),
    Btype(B),
}

impl<V, A, B> Deref for DualCell<V, A, B>
where
    A: Deref<Target = V>,
    B: Deref<Target = V>,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Atype(v) => v.deref(),
            Self::Btype(v) => v.deref(),
        }
    }
}

impl<V, A, B> DerefMut for DualCell<V, A, B>
where
    A: Deref<Target = V> + DerefMut,
    B: Deref<Target = V> + DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Atype(v) => v.deref_mut(),
            Self::Btype(v) => v.deref_mut(),
        }
    }
}

impl<K, V, C1, C2, U1, U2, W1, W2>
    Get<K, V, DualCell<V, U1, U2>, DualCell<V, W1, W2>> for Stack<C1, C2>
where
    U1: Deref<Target = V>,
    U2: Deref<Target = V>,
    W1: Deref<Target = V> + DerefMut,
    W2: Deref<Target = V> + DerefMut,
    C1: Get<K, V, U1, W1>,
    C2: Get<K, V, U2, W2>,
{
    fn get<'a>(&'a self, key: &K) -> Option<DualCell<V, U1, U2>> {
        match self.l1.get(key) {
            Some(x) => Some(DualCell::Atype(x)),
            None => match self.l2.get(key) {
                None => None,
                Some(x) => Some(DualCell::Btype(x)),
            },
        }
    }

    fn get_mut<'a>(&'a mut self, key: &K) -> Option<DualCell<V, W1, W2>> {
        match self.l1.get_mut(key) {
            Some(x) => Some(DualCell::Atype(x)),
            None => match self.l2.get_mut(key) {
                None => None,
                Some(x) => Some(DualCell::Btype(x)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;
    use crate::container::Vector;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        test_building_block(Stack::new(Vector::new(0), Vector::new(0)));
        test_building_block(Stack::new(Vector::new(0), Vector::new(10)));
        test_building_block(Stack::new(Vector::new(10), Vector::new(0)));
        test_building_block(Stack::new(Vector::new(10), Vector::new(100)));
    }
}

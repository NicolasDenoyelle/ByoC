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
/// // This building block tries to make room in the first layer
/// // to push new elements, which means that "third" will be in the first
/// // layer. "second" is moved up to the second layer and "first" is
/// // popped.
/// let victim = cache.push(vec![("third", 2)]).pop().unwrap();
/// assert_eq!(victim.0, "first");
///
/// // Pop takes elements from the second layer and then from the first
/// // layer.
/// let (k, v) = cache.pop(1).pop().unwrap();
/// assert_eq!(k, "second");
/// let (k, v) = cache.pop(1).pop().unwrap();
/// assert_eq!(k, "third");
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
        let l1_capacity = self.l1.capacity();
        let l1_count = self.l1.count();

        let mut l1_pop = if elements.len() <= (l1_capacity - l1_count) {
            Vec::new()
        } else if elements.len() <= l1_capacity {
            let pop_count = elements.len() + l1_count - l1_capacity;
            self.l1.pop(pop_count)
        } else {
            self.l1.flush().collect()
        };
				let mut elements = self.l1.push(elements);
				elements.append(&mut l1_pop);

				if elements.len() == 0 {
						return elements;
				}

				let l2_capacity = self.l2.capacity();
        let l2_count = self.l2.count();
        let mut l2_pop = if elements.len() <= (l2_capacity - l2_count) {
            Vec::new()
        } else if elements.len() <= l2_capacity {
            let pop_count = elements.len() + l2_count - l2_capacity;
            self.l2.pop(pop_count)
        } else {
            self.l2.flush().collect()
        };
				let mut elements = self.l2.push(elements);
				elements.append(&mut l2_pop);

				elements
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

impl<'b, K, V, C1, C2, U1, U2, W1, W2>
    Get<K, V, DualCell<V, U1, U2>, DualCell<V, W1, W2>> for Stack<C1, C2>
where
    K: 'b,
    V: 'b,
    U1: Deref<Target = V>,
    U2: Deref<Target = V>,
    W1: Deref<Target = V> + DerefMut,
    W2: Deref<Target = V> + DerefMut,
    C1: Get<K, V, U1, W1> + BuildingBlock<'b, K, V>,
    C2: Get<K, V, U2, W2> + BuildingBlock<'b, K, V>,
{
    unsafe fn get(&self, key: &K) -> Option<DualCell<V, U1, U2>> {
        match self.l1.get(key) {
            Some(x) => Some(DualCell::Atype(x)),
            None => match self.l2.get(key) {
                None => None,
                Some(x) => Some(DualCell::Btype(x)),
            },
        }
    }

    unsafe fn get_mut(&mut self, key: &K) -> Option<DualCell<V, W1, W2>> {
        // If key is in l1, we can return it.
        if let Some(x) = self.l1.get_mut(key) {
            return Some(DualCell::Atype(x));
        }

        // If value is not in l2, then we return None.
        // Else we will try to promote it in l1.
        let (k, v) = match self.l2.take(key) {
            None => return None,
            Some(x) => x,
        };

        // We push the value in l1. If it does not pop, we return it.
        let (k, v) = match self.l1.push(vec![(k, v)]).pop() {
            None => {
                return Some(DualCell::Atype(self.l1.get_mut(key).expect(
                    "Element inserted in l1 cannot be retrieved",
                )))
            }
            Some(x) => x,
        };

        // The value popped...
        // We try to make room in l1 by popping something..
        let (k1, v1) = match self.l1.pop(1).pop() {
            // L1 popped an item.
            Some(item) => item,
            // L1 can't pop, we have no choice but to use l2.
            None => {
                // Fails if cannot reinsert an element in l2 that used to be
                // in l2.
                assert!(self.l2.push(vec![(k, v)]).pop().is_none());
                return Some(DualCell::Btype(
                    self.l2
                        .get_mut(key)
                        .expect("Key inside container not found"),
                ));
            }
        };

        // Now there should be room in l1 and l2.
        // Let's try to put the desired key in l1
        let ((k, v), (k1, v1)) = match self.l1.push(vec![(k, v)]).pop() {
            // push worked, now we push in l2 and return the key in l1.
            None => {
                match self.l2.push(vec![(k1, v1)]).pop() {
                    None => {
                        return Some(DualCell::Atype(
                            self.l1
                                .get_mut(key)
                                .expect("Key inside container not found"),
                        ))
                    }
                    // Push in l2 did not work. We have to back track to the
                    // initial situation and return the key/value from L2.
                    Some((k1, v1)) => (
                        self.l1
                            .take(key)
                            .expect("Key inside container not found"),
                        (k1, v1),
                    ),
                }
            }

            // Push in l1 did not work. We reinsert element where they were
            // and we have to use l2.
            Some((k, v)) => ((k, v), (k1, v1)),
        };

        // Push did not work. We reinsert element where they were
        // and we have to use l2.
        assert!(self.l1.push(vec![(k1, v1)]).pop().is_none());
        assert!(self.l2.push(vec![(k, v)]).pop().is_none());
        return Some(DualCell::Btype(
            self.l2
                .get_mut(key)
                .expect("Key inside container not found"),
        ));
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Stack;
    use crate::container::Vector;
    use crate::tests::{test_building_block, test_get};

    #[test]
    fn building_block() {
        test_building_block(Stack::new(Vector::new(0), Vector::new(0)));
        test_building_block(Stack::new(Vector::new(0), Vector::new(10)));
        test_building_block(Stack::new(Vector::new(10), Vector::new(0)));
        test_building_block(Stack::new(Vector::new(10), Vector::new(100)));
    }

    #[test]
    fn get() {
        test_get(Stack::new(Vector::new(0), Vector::new(0)));
        test_get(Stack::new(Vector::new(0), Vector::new(10)));
        test_get(Stack::new(Vector::new(10), Vector::new(0)));
        test_get(Stack::new(Vector::new(10), Vector::new(100)));
    }
}

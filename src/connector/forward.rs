use crate::{BuildingBlock, Get};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// BuildingBlock Stack
//------------------------------------------------------------------------//

/// Connect two building blocks with key/values going forward from the
/// left container to the right container.
///
/// This building block typically implements the policy for how data is
/// moved from one level to another in a multi-level cache, where the left
/// building block is used as a cache for the right building block.
/// Elements transfer from one side to another only happen when the cache
/// is accessed in a mutable fashion, mainly when calling
/// [`push()`](../trait.BuildingBlock.html#tymethod.push) or
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop).
///
/// See `BuildingBlock` and `Get` implementations details for more
/// information on data movement implementation.
///
/// ## Examples
///
/// ```
/// use cache::{BuildingBlock, Get};
/// use cache::connector::Forward;
/// use cache::container::Array;
///
/// // Create cache
/// let mut left = Array::new(2);
/// let mut right = Array::new(4);
/// let mut cache = Forward::new(left, right);
/// // [[][]]
///
/// // Populate left side.
/// assert!(cache.push(vec![("first", 1), ("second", 0)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][]]
///
/// // Left side is full. Next push will move the highest values
/// // from the left side to the right side to make room for the new
/// // value in the left side.
/// assert!(cache.push(vec![("third", 3)]).pop().is_none());
/// // [[("second", 0), ("third", 3)][("first", 1)]]
///
/// // At this point, "second" and "third" are in left side and
/// // "first" is in the right side.
/// // Pop operation removes elements from the right side first
/// // then the left side.
/// assert_eq!(cache.pop(1).pop().unwrap().0, "first");
/// // [[("second", 0), ("third", 3)][]]
///
/// // We reinsert "first". As a result "third" moves to the right side
/// // because the associated value is the highest on this side of the
/// // cache.
/// assert!(cache.push(vec![("first", 1)]).pop().is_none());
/// // [[("first", 1), ("second", 0)][("third", 3)]]
///
/// // Accessing a (mutable) value promotes it to the left side.
/// // If we access "third", it moves to the left side while "first",
/// // will go back to the right side because it is the highest value.
/// let mut val = unsafe { cache.get_mut(&"third") };
/// // [[("second", 0), ("third", 3)][("first", 1)]]
/// assert_eq!(cache.pop(1).pop().unwrap().0, "first")
/// // [[("second", 0), ("third", 3)][]]
/// ```
pub struct Forward<L, R> {
    left: L,
    right: R,
}

impl<L, R> Forward<L, R> {
    /// Construct a Forward Cache.
    pub fn new(left: L, right: R) -> Self {
        Forward {
            left: left,
            right: right,
        }
    }
}

impl<'a, K: 'a, V: 'a, L, R> BuildingBlock<'a, K, V> for Forward<L, R>
where
    L: BuildingBlock<'a, K, V>,
    R: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.left.capacity() + self.right.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.left.flush().chain(self.right.flush()))
    }

    /// The left side is looked first and if the key is not
    /// found, it is searched in the right side.
    fn contains(&self, key: &K) -> bool {
        if self.left.contains(key) {
            true
        } else {
            self.right.contains(key)
        }
    }

    fn count(&self) -> usize {
        self.left.count() + self.right.count()
    }

    /// Take the matching key/value pair out of the container.
    /// The left side is looked first and if the key is not
    /// found, it is searched in the right side.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.left.take(key) {
            Some(x) => Some(x),
            None => self.right.take(key),
        }
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// Pop will remove values from the right side all it can.
    /// If there were less than `n` values in the right side,
    /// then more values from the left side are popped.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut v = self.right.pop(n);

        if v.len() < n {
            v.append(&mut self.left.pop(n - v.len()));
        }
        v
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    ///
    /// Push will make room (pop) from the left side to the right side to
    /// fit as many new `elements` as possible. If there is more elements
    /// than capacity in the left side, the left side is flushed to the
    /// right side. At this point, everything that overflows the right side
    /// will be returned.
    /// Once room has been made, `elements` are inserted to the left.
    /// If new elements pop in the process, they are inserted to the
    /// right side. If elements pop again on the right side, they are
    /// also returned.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let left_capacity = self.left.capacity();
        let left_count = self.left.count();

        let mut left_pop = if elements.len()
            <= (left_capacity - left_count)
        {
            Vec::new()
        } else if elements.len() <= left_capacity {
            let pop_count = elements.len() + left_count - left_capacity;
            self.left.pop(pop_count)
        } else {
            self.left.flush().collect()
        };
        let mut elements = self.left.push(elements);
        elements.append(&mut left_pop);

        if elements.len() == 0 {
            return elements;
        }

        let right_capacity = self.right.capacity();
        let right_count = self.right.count();
        let mut right_pop = if elements.len()
            <= (right_capacity - right_count)
        {
            Vec::new()
        } else if elements.len() <= right_capacity {
            let pop_count = elements.len() + right_count - right_capacity;
            self.right.pop(pop_count)
        } else {
            self.right.flush().collect()
        };
        let mut elements = self.right.push(elements);
        elements.append(&mut right_pop);

        elements
    }
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

/// Cell wrapping an element in a [`Forward`](struct.Forward.html)
/// building block.
///
/// This cell can wrap both read-only and read-write elements.
/// The element may come from the left or right side of the `Forward`
/// container. Safety of accessing this cell depends on the safety of
/// accessing elements on both sides. This may vary depending on
/// the element being is read-only or being accessible for writing.
pub enum ForwardCell<V, L, R>
where
    L: Deref<Target = V>,
    R: Deref<Target = V>,
{
    Ltype(L),
    Rtype(R),
}

impl<V, L, R> Deref for ForwardCell<V, L, R>
where
    L: Deref<Target = V>,
    R: Deref<Target = V>,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ltype(v) => v.deref(),
            Self::Rtype(v) => v.deref(),
        }
    }
}

impl<V, L, R> DerefMut for ForwardCell<V, L, R>
where
    L: Deref<Target = V> + DerefMut,
    R: Deref<Target = V> + DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Ltype(v) => v.deref_mut(),
            Self::Rtype(v) => v.deref_mut(),
        }
    }
}

impl<'b, K, V, L, R, LU, RU, LW, RW>
    Get<K, V, ForwardCell<V, LU, RU>, ForwardCell<V, LW, RW>>
    for Forward<L, R>
where
    K: 'b,
    V: 'b,
    LU: Deref<Target = V>,
    RU: Deref<Target = V>,
    LW: Deref<Target = V> + DerefMut,
    RW: Deref<Target = V> + DerefMut,
    L: Get<K, V, LU, LW> + BuildingBlock<'b, K, V>,
    R: Get<K, V, RU, RW> + BuildingBlock<'b, K, V>,
{
    unsafe fn get(&self, key: &K) -> Option<ForwardCell<V, LU, RU>> {
        match self.left.get(key) {
            Some(x) => Some(ForwardCell::Ltype(x)),
            None => match self.right.get(key) {
                None => None,
                Some(x) => Some(ForwardCell::Rtype(x)),
            },
        }
    }

    /// Get a smart pointer to a mutable value inside the container.
    ///
    /// The element will be searched first in the left side.
    /// If it is not found, it is searched in the right side.
    /// If it is found in the right side, we try to make room
    /// in the left side to move it there.
    /// If the left side can't pop, the found element is reinserted
    /// on the right side and returned from there.
    /// If the left side can pop, the element is inserted in the left side
    /// in lieu of a victim and the victim is inserted on the right side.
    /// If the insertion of the victim fails on the right side,
    /// we take back the element in the left side and put it back in the
    /// right side, while the victim goes back in the left side.
    unsafe fn get_mut(
        &mut self,
        key: &K,
    ) -> Option<ForwardCell<V, LW, RW>> {
        // If key is in left, we can return it.
        if let Some(x) = self.left.get_mut(key) {
            return Some(ForwardCell::Ltype(x));
        }

        // If value is not in right, then we return None.
        // Else we will try to promote it in left.
        let (k, v) = match self.right.take(key) {
            None => return None,
            Some(x) => x,
        };

        // We push the value in left. If it does not pop, we return it.
        let (k, v) = match self.left.push(vec![(k, v)]).pop() {
            None => {
                return Some(ForwardCell::Ltype(
                    self.left.get_mut(key).expect(
                        "Element inserted in left cannot be retrieved",
                    ),
                ))
            }
            Some(x) => x,
        };

        // The value popped...
        // We try to make room in left by popping something..
        let (k1, v1) = match self.left.pop(1).pop() {
            // LEFT popped an item.
            Some(item) => item,
            // LEFT can't pop, we have no choice but to use right.
            None => {
                // Fails if cannot reinsert an element in right that used to be
                // in right.
                assert!(self.right.push(vec![(k, v)]).pop().is_none());
                return Some(ForwardCell::Rtype(
                    self.right
                        .get_mut(key)
                        .expect("Key inside container not found"),
                ));
            }
        };

        // Now there should be room in left and right.
        // Let's try to put the desired key in left
        let ((k, v), (k1, v1)) = match self.left.push(vec![(k, v)]).pop() {
            // push worked, now we push in right and return the key in left.
            None => {
                match self.right.push(vec![(k1, v1)]).pop() {
                    None => {
                        return Some(ForwardCell::Ltype(
                            self.left
                                .get_mut(key)
                                .expect("Key inside container not found"),
                        ))
                    }
                    // Push in right did not work. We have to back track to the
                    // initial situation and return the key/value from RIGHT.
                    Some((k1, v1)) => (
                        self.left
                            .take(key)
                            .expect("Key inside container not found"),
                        (k1, v1),
                    ),
                }
            }

            // Push in left did not work. We reinsert element where they were
            // and we have to use right.
            Some((k, v)) => ((k, v), (k1, v1)),
        };

        // Push did not work. We reinsert element where they were
        // and we have to use right.
        assert!(self.left.push(vec![(k1, v1)]).pop().is_none());
        assert!(self.right.push(vec![(k, v)]).pop().is_none());
        return Some(ForwardCell::Rtype(
            self.right
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
    use super::Forward;
    use crate::container::Array;
    use crate::tests::{test_building_block, test_get};

    #[test]
    fn building_block() {
        test_building_block(Forward::new(Array::new(0), Array::new(0)));
        test_building_block(Forward::new(Array::new(0), Array::new(10)));
        test_building_block(Forward::new(Array::new(10), Array::new(0)));
        test_building_block(Forward::new(Array::new(10), Array::new(100)));
    }

    #[test]
    fn get() {
        test_get(Forward::new(Array::new(0), Array::new(0)));
        test_get(Forward::new(Array::new(0), Array::new(10)));
        test_get(Forward::new(Array::new(10), Array::new(0)));
        test_get(Forward::new(Array::new(10), Array::new(100)));
    }
}

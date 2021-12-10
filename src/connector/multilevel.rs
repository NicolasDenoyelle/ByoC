use crate::{BuildingBlock, Get, GetMut, Prefetch};
use std::marker::PhantomData;
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
/// use cache::{BuildingBlock, GetMut};
/// use cache::connector::Multilevel;
/// use cache::container::Array;
///
/// // Create cache
/// let mut left = Array::new(2);
/// let mut right = Array::new(4);
/// let mut cache = Multilevel::new(left, right);
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
pub struct Multilevel<K, V, L, R> {
    left: L,
    right: R,
    unused: PhantomData<(K, V)>,
}

impl<K, V, L, R> Multilevel<K, V, L, R> {
    /// Construct a Multilevel Cache.
    pub fn new(left: L, right: R) -> Self {
        Multilevel {
            left: left,
            right: right,
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, L, R> BuildingBlock<'a, K, V> for Multilevel<K, V, L, R>
where
    K: 'a,
    V: 'a,
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

/// Cell wrapping an element in a [`Multilevel`](struct.Multilevel.html)
/// building block.
///
/// This cell can wrap both read-only and read-write elements.
/// The element may come from the left or right side of the `Multilevel`
/// container. Safety of accessing this cell depends on the safety of
/// accessing elements on both sides. This may vary depending on
/// the element being is read-only or being accessible for writing.
pub enum MultilevelCell<V, L, R>
where
    L: Deref<Target = V>,
    R: Deref<Target = V>,
{
    Ltype(L),
    Rtype(R),
}

impl<V, L, R> Deref for MultilevelCell<V, L, R>
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

impl<V, L, R> DerefMut for MultilevelCell<V, L, R>
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

impl<'b, K, V, L, R, LU, RU> Get<K, V, MultilevelCell<V, LU, RU>>
    for Multilevel<K, V, L, R>
where
    K: 'b,
    V: 'b,
    LU: Deref<Target = V>,
    RU: Deref<Target = V>,
    L: Get<K, V, LU> + BuildingBlock<'b, K, V>,
    R: Get<K, V, RU> + BuildingBlock<'b, K, V>,
{
    unsafe fn get(&self, key: &K) -> Option<MultilevelCell<V, LU, RU>> {
        match self.left.get(key) {
            Some(x) => Some(MultilevelCell::Ltype(x)),
            None => match self.right.get(key) {
                None => None,
                Some(x) => Some(MultilevelCell::Rtype(x)),
            },
        }
    }
}

impl<'b, K, V, L, R, LW> GetMut<K, V, LW> for Multilevel<K, V, L, R>
where
    K: 'b,
    V: 'b,
    LW: Deref<Target = V> + DerefMut,
    L: GetMut<K, V, LW> + BuildingBlock<'b, K, V>,
    R: BuildingBlock<'b, K, V>,
{
    /// Get a smart pointer to a mutable value inside the container.
    ///
    /// The element will be searched first in the left side.
    /// If it is not found, it is searched in the right side.
    /// If it is found in the right side, we try to make room
    /// in the left side to move it there.
    /// If the left side can't pop, None is returned even though
    /// the value is in the building block.
    /// If the left side can pop, the element is inserted in the left side
    /// in lieu of a victim and the victim is inserted on the right side.
    /// If one of these insertions fail, we back track to the initial
    /// building block state and None is returned even though the value is
    /// in the building block.
    /// If they succeed or if the element was already on the left side,
    /// we return the value from the left side.
    unsafe fn get_mut(&mut self, key: &K) -> Option<LW> {
        // If key is in left, we can return it.
        if let Some(x) = self.left.get_mut(key) {
            return Some(x);
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
                return Some(self.left.get_mut(key).expect(
                    "Element inserted in left cannot be retrieved",
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
                // in right and we return None.
                assert!(self.right.push(vec![(k, v)]).pop().is_none());
                return None;
            }
        };

        // Now there should be room in left and right.
        // Let's try to put the desired key in left
        let ((k, v), (k1, v1)) = match self.left.push(vec![(k, v)]).pop() {
            // push worked, now we push in right and return the key in left.
            None => {
                match self.right.push(vec![(k1, v1)]).pop() {
                    None => {
                        return Some(
                            self.left
                                .get_mut(key)
                                .expect("Key inside container not found"),
                        )
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
        return None;
    }
}

//------------------------------------------------------------------------//
// Prefetch trait
//------------------------------------------------------------------------//

impl<'a, K, V, L, R> Prefetch<'a, K, V> for Multilevel<K, V, L, R>
where
    K: 'a + Ord,
    V: 'a,
    L: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
    R: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
{
    /// Multilevel prefetch implementation moves matching keys
    /// in the right side into the left side.
    /// This is achieved by calling the
    /// [`push()`](struct.Multilevel.html#method.push) method after retrieving
    /// elements from the right side.
    fn prefetch(&mut self, mut keys: Vec<K>) {
        // Then right side.
        let matches = self.right.take_multiple(&mut keys);

        // Finally insert matches.
        // Reinsertion must work because we the container still has the same
        // number of elements.
        if matches.len() > 0 {
            assert!(self.push(matches).pop().is_none());
        }
    }

    /// This method will take matching keys on the left side then on
    /// the right side.
    /// Matching keys found on the left side are not searched on the right
    /// side.
    /// Input `keys` is updated as a side effect to contain
    /// only non matching keys.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.sort();

        let mut left = self.left.take_multiple(keys);

        // Remove matches from keys before querying on the right side.
        for (k, _) in left.iter() {
            match keys.binary_search(k) {
                Ok(i) => {
                    keys.remove(i);
                }
                Err(_) => {}
            }
        }

        let mut right = self.right.take_multiple(keys);

        // Remove matching keys in case these keys are used in other
        // calls to take_multiple.
        for (k, _) in right.iter() {
            match keys.binary_search(k) {
                Ok(i) => {
                    keys.remove(i);
                }
                Err(_) => {}
            }
        }

        // Return final matches.
        left.append(&mut right);
        left
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Multilevel;
    use crate::container::Array;
    use crate::tests::{
        test_building_block, test_get, test_get_mut, test_prefetch,
    };

    #[test]
    fn building_block() {
        test_building_block(Multilevel::new(Array::new(0), Array::new(0)));
        test_building_block(Multilevel::new(
            Array::new(0),
            Array::new(10),
        ));
        test_building_block(Multilevel::new(
            Array::new(10),
            Array::new(0),
        ));
        test_building_block(Multilevel::new(
            Array::new(10),
            Array::new(100),
        ));
    }

    #[test]
    fn get() {
        test_get(Multilevel::new(Array::new(0), Array::new(0)));
        test_get(Multilevel::new(Array::new(0), Array::new(10)));
        test_get(Multilevel::new(Array::new(10), Array::new(0)));
        test_get(Multilevel::new(Array::new(10), Array::new(100)));
        test_get_mut(Multilevel::new(Array::new(10), Array::new(0)));
        test_get_mut(Multilevel::new(Array::new(10), Array::new(100)));
    }

    #[test]
    fn prefetch() {
        test_prefetch(Multilevel::new(Array::new(0), Array::new(0)));
        test_prefetch(Multilevel::new(Array::new(0), Array::new(10)));
        test_prefetch(Multilevel::new(Array::new(10), Array::new(0)));
        test_prefetch(Multilevel::new(Array::new(10), Array::new(100)));
    }
}

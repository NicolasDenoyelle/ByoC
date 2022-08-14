use super::Multilevel;
use crate::{BuildingBlock, Get, GetMut};
use std::ops::{Deref, DerefMut};

/// Cell wrapping an element in a `Multilevel` building block.
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
            None => self.right.get(key).map(MultilevelCell::Rtype),
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
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Multilevel;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        test_get(Multilevel::new(Array::new(0), Array::new(0)));
        test_get(Multilevel::new(Array::new(0), Array::new(10)));
        test_get(Multilevel::new(Array::new(10), Array::new(0)));
        test_get(Multilevel::new(Array::new(10), Array::new(100)));
        test_get_mut(Multilevel::new(Array::new(10), Array::new(0)));
        test_get_mut(Multilevel::new(Array::new(10), Array::new(100)));
    }
}

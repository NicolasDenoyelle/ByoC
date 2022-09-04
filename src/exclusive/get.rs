use super::Exclusive;
use crate::utils::get::LifeTimeGuard;
use crate::{BuildingBlock, Get, GetMut};
use std::ops::{Deref, DerefMut};

/// Cell wrapping an element in a `Exclusive` building block.
///
/// This cell can wrap both read-only and read-write elements.
/// The element may come from the front or back of the `Exclusive`
/// container. Safety of accessing this cell depends on the safety of
/// accessing elements on both sides. This may vary depending on
/// the element being is read-only or being accessible for writing.
pub enum ExclusiveCell<V, L, R>
where
    L: Deref<Target = V>,
    R: Deref<Target = V>,
{
    Ltype(L),
    Rtype(R),
}

impl<V, L, R> Deref for ExclusiveCell<V, L, R>
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

impl<V, L, R> DerefMut for ExclusiveCell<V, L, R>
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

impl<'b, K, V, L, R> Get<K, V> for Exclusive<K, V, L, R>
where
    K: 'b,
    V: 'b,
    L: Get<K, V> + BuildingBlock<'b, K, V>,
    R: Get<K, V> + BuildingBlock<'b, K, V>,
{
    type Target = ExclusiveCell<V, L::Target, R::Target>;

    fn get(&self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        match self.front.get(key) {
            Some(x) => {
                Some(LifeTimeGuard::new(ExclusiveCell::Ltype(x.unwrap())))
            }
            None => self.back.get(key).map(|y| {
                LifeTimeGuard::new(ExclusiveCell::Rtype(y.unwrap()))
            }),
        }
    }
}

impl<'b, K, V, L, R> GetMut<K, V> for Exclusive<K, V, L, R>
where
    K: 'b,
    V: 'b,
    L: GetMut<K, V> + BuildingBlock<'b, K, V>,
    R: GetMut<K, V> + BuildingBlock<'b, K, V>,
{
    type Target = ExclusiveCell<V, L::Target, R::Target>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        match self.front.get_mut(key) {
            Some(x) => {
                Some(LifeTimeGuard::new(ExclusiveCell::Ltype(x.unwrap())))
            }
            None => self.back.get_mut(key).map(|y| {
                LifeTimeGuard::new(ExclusiveCell::Rtype(y.unwrap()))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Exclusive;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        test_get(Exclusive::new(Array::new(0), Array::new(0)));
        test_get(Exclusive::new(Array::new(0), Array::new(10)));
        test_get(Exclusive::new(Array::new(10), Array::new(0)));
        test_get(Exclusive::new(Array::new(10), Array::new(100)));
        test_get_mut(Exclusive::new(Array::new(10), Array::new(0)));
        test_get_mut(Exclusive::new(Array::new(10), Array::new(100)));
    }
}

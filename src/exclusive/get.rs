use super::Exclusive;
use crate::utils::get::LifeTimeGuard;
use crate::{BuildingBlock, Get, GetMut};

impl<K, V, L, R> Get<K, V> for Exclusive<K, V, L, R>
where
    L: Get<K, V> + BuildingBlock<K, V>,
    R: BuildingBlock<K, V>,
{
    type Target = L::Target;

    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        // Lookup in the front stage of the cache.
        // If element is there return it.
        if self.front.contains(key) {
            return self.front.get(key);
        };

        match self.downgrade(key) {
            true => self.front.get(key),
            false => None,
        }
    }
}

impl<K, V, L, R> GetMut<K, V> for Exclusive<K, V, L, R>
where
    L: GetMut<K, V> + BuildingBlock<K, V>,
    R: BuildingBlock<K, V>,
{
    type Target = L::Target;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        // Lookup in the front stage of the cache.
        // If element is there return it.
        if self.front.contains(key) {
            return self.front.get_mut(key);
        };

        match self.downgrade(key) {
            true => self.front.get_mut(key),
            false => None,
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
        test_get(Exclusive::new(Array::new(10), Array::new(0)));
        test_get(Exclusive::new(Array::new(10), Array::new(100)));
        test_get_mut(Exclusive::new(Array::new(10), Array::new(0)));
        test_get_mut(Exclusive::new(Array::new(10), Array::new(100)));
    }
}

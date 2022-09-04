use super::Associative;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};
use std::hash::{Hash, Hasher};

impl<K, V, C, H> Get<K, V> for Associative<C, H>
where
    K: Hash + Clone,
    H: Hasher + Clone,
    C: Get<K, V>,
{
    type Target = C::Target;
    fn get(&self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }
}

impl<K, V, C, H> GetMut<K, V> for Associative<C, H>
where
    K: Hash + Clone,
    H: Hasher + Clone,
    C: GetMut<K, V>,
{
    type Target = C::Target;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let i = self.set(key.clone());
        self.containers[i].get_mut(key)
    }
}

#[cfg(test)]
mod tests {
    use super::Associative;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn get() {
        test_get(Associative::new(
            vec![Array::new(5); 10],
            DefaultHasher::new(),
        ));
        test_get_mut(Associative::new(
            vec![Array::new(5); 10],
            DefaultHasher::new(),
        ));
    }
}

use super::Associative;
use crate::{Get, GetMut};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

impl<K, V, U, C, H> Get<K, V, U> for Associative<C, H>
where
    K: Hash + Clone,
    U: Deref<Target = V>,
    H: Hasher + Clone,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }
}

impl<K, V, W, C, H> GetMut<K, V, W> for Associative<C, H>
where
    K: Hash + Clone,
    W: DerefMut<Target = V>,
    H: Hasher + Clone,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
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

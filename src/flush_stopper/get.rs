use super::FlushStopper;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};

impl<K, V, C: Get<K, V>> Get<K, V> for FlushStopper<C> {
    type Target = C::Target;
    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.container.get(key)
    }
}

impl<K, V, C: GetMut<K, V>> GetMut<K, V> for FlushStopper<C> {
    type Target = C::Target;
    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.container.get_mut(key)
    }
}

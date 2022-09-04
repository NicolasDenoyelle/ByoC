use super::Batch;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};

impl<K, V, C> Get<K, V> for Batch<C>
where
    C: Get<K, V>,
{
    type Target = C::Target;
    fn get(&self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.bb.iter().find_map(|c| c.get(key))
    }
}

impl<K, V, C> GetMut<K, V> for Batch<C>
where
    C: GetMut<K, V>,
{
    type Target = C::Target;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.bb.iter_mut().find_map(|c| c.get_mut(key))
    }
}

#[cfg(test)]
mod tests {
    use super::Batch;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        test_get(Batch::<Array<(u16, u32)>>::new());
        test_get(Batch::from([Array::new(0)]));
        test_get(Batch::from([Array::new(0), Array::new(0)]));
        test_get(Batch::from([Array::new(0), Array::new(10)]));
        test_get(Batch::from([Array::new(10), Array::new(0)]));
        test_get(Batch::from([Array::new(10), Array::new(10)]));
    }

    #[test]
    fn get_mut() {
        test_get_mut(Batch::<Array<(u16, u32)>>::new());
        test_get_mut(Batch::from([Array::new(0)]));
        test_get_mut(Batch::from([Array::new(0), Array::new(0)]));
        test_get_mut(Batch::from([Array::new(0), Array::new(10)]));
        test_get_mut(Batch::from([Array::new(10), Array::new(0)]));
        test_get_mut(Batch::from([Array::new(10), Array::new(10)]));
    }
}

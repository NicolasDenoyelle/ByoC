use super::Batch;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};

impl<K, V, C> Get<K, V> for Batch<C>
where
    C: Get<K, V>,
{
    type Target = C::Target;
    /// Get a read-only smart pointer to a value inside the container.
    ///
    /// [`get()`](trait.Get.html#method.get) method works similarly
    /// as [`take()`](trait.BuildingBlock.html#tymethod.take) method.
    /// It iterates batches from the front to the back and stops at the first
    /// match.
    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.bb.iter_mut().find_map(|c| c.get(key))
    }
}

impl<K, V, C> GetMut<K, V> for Batch<C>
where
    C: GetMut<K, V>,
{
    type Target = C::Target;

    /// Get a smart pointer to a mutable value inside the container.
    ///
    /// [`get_mut()`](trait.GetMut.html#method.get_mut) method works similarly
    /// as [`take()`](trait.BuildingBlock.html#tymethod.take) method.
    /// It iterates batches from the front to the back and stops at the first
    /// match.
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

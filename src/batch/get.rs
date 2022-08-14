use super::Batch;
use crate::{Get, GetMut};
use std::ops::{Deref, DerefMut};

impl<K, V, U, C> Get<K, V, U> for Batch<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        self.bb.iter().find_map(|c| c.get(key))
    }
}

impl<K, V, W, C> GetMut<K, V, W> for Batch<C>
where
    W: DerefMut<Target = V>,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
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

use crate::policy::{Ordered, Reference, ReferenceFactory};
use crate::{Get, GetMut, Policy};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A Cell wrapping elements borrowed from inside a `Policy` building block.
///
/// It can be dereferenced through the wrapped building block element cell
/// to obtain original value.
///
/// ## Safety:
///
/// The safety of using this cell depends on the safety of using the wrapped
/// element cell.
pub struct PolicyCell<V, R, U>
where
    R: Reference<V>,
    U: Deref<Target = R>,
{
    item: U,
    unused: PhantomData<V>,
}

impl<V, R, U> Deref for PolicyCell<V, R, U>
where
    R: Reference<V>,
    U: Deref<Target = R>,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.item.deref().get()
    }
}

impl<V, R, W> DerefMut for PolicyCell<V, R, W>
where
    R: Reference<V>,
    W: Deref<Target = R> + DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.deref_mut().get_mut()
    }
}

impl<K, V, R, U, F, C> Get<K, V, PolicyCell<V, R, U>>
    for Policy<C, V, R, F>
where
    R: Reference<V>,
    U: Deref<Target = R>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: Get<K, R, U> + Ordered<R>,
{
    unsafe fn get(&self, key: &K) -> Option<PolicyCell<V, R, U>> {
        self.container.get(key).map(|x| PolicyCell {
            item: x,
            unused: PhantomData,
        })
    }
}

impl<K, V, R, W, F, C> GetMut<K, V, PolicyCell<V, R, W>>
    for Policy<C, V, R, F>
where
    R: Reference<V>,
    W: DerefMut<Target = R>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: GetMut<K, R, W> + Ordered<R>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<PolicyCell<V, R, W>> {
        self.container.get_mut(key).map(|x| PolicyCell {
            item: x,
            unused: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::policy::Default;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        for i in [0usize, 10usize, 100usize] {
            test_get(Policy::new(Array::new(i), Default {}));
            test_get_mut(Policy::new(Array::new(i), Default {}));
        }
    }
}
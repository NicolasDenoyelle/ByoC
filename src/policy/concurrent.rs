use super::{Ordered, ReferenceFactory};
use crate::{Concurrent, Policy};
use std::marker::PhantomData;

unsafe impl<C, V, F> Send for Policy<C, V, F>
where
    F: ReferenceFactory<V> + Send,
    C: Ordered<F::Item> + Send,
{
}

unsafe impl<C, V, F> Sync for Policy<C, V, F>
where
    F: ReferenceFactory<V> + Sync,
    C: Ordered<F::Item> + Sync,
{
}

impl<'a, V, C, F> Concurrent for Policy<C, V, F>
where
    V: 'a,
    F: ReferenceFactory<V> + Clone + Send + Sync,
    C: Ordered<F::Item> + Concurrent,
{
    fn clone(&self) -> Self {
        Policy {
            container: Concurrent::clone(&self.container),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

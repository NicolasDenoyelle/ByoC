use crate::policy::{Ordered, Reference, ReferenceFactory};
use crate::{Concurrent, Policy};
use std::marker::PhantomData;

unsafe impl<C, V, R, F> Send for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Send,
    C: Ordered<R> + Send,
{
}

unsafe impl<C, V, R, F> Sync for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Sync,
    C: Ordered<R> + Sync,
{
}

impl<'a, V, C, R, F> Concurrent for Policy<C, V, R, F>
where
    V: 'a,
    R: 'a + Reference<V>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: Ordered<R> + Concurrent,
{
    fn clone(&self) -> Self {
        Policy {
            container: Concurrent::clone(&self.container),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

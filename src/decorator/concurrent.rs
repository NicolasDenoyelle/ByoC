use super::DecorationFactory;
use crate::{Concurrent, Decorator};
use std::marker::PhantomData;

unsafe impl<C, V, F> Send for Decorator<C, V, F>
where
    F: DecorationFactory<V> + Send,
    C: Send,
{
}

unsafe impl<C, V, F> Sync for Decorator<C, V, F>
where
    F: DecorationFactory<V> + Sync,
    C: Sync,
{
}

impl<'a, V, C, F> Concurrent for Decorator<C, V, F>
where
    V: 'a,
    F: DecorationFactory<V> + Clone + Send + Sync,
    C: Concurrent,
{
    fn clone(&self) -> Self {
        Decorator {
            container: Concurrent::clone(&self.container),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

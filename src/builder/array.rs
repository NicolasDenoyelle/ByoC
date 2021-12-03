use crate::builder::{Builder, ForwardBuilder, PolicyBuilder};
use crate::container::Array;
use crate::policy::{Reference, ReferenceFactory};
use std::marker::PhantomData;

pub struct ArrayBuilder<T> {
    capacity: usize,
    unused: PhantomData<T>,
}

impl<K, V> ArrayBuilder<(K, V)> {
    pub fn forward<R, RB: Builder<R>>(
        self,
    ) -> ForwardBuilder<Array<(K, V)>, ArrayBuilder<(K, V)>, R, RB> {
        ForwardBuilder::new(self)
    }

    pub fn with_policy<R: Reference<V>, F: ReferenceFactory<V, R>>(
        self,
        policy: F,
    ) -> PolicyBuilder<Array<(K, V)>, V, R, F, ArrayBuilder<(K, V)>> {
        PolicyBuilder::new(self, policy)
    }
}

impl<T> Builder<Array<T>> for ArrayBuilder<T> {
    fn build(self) -> Array<T> {
        Array::new(self.capacity)
    }
}

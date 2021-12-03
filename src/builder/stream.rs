use crate::builder::{Builder, ForwardBuilder, PolicyBuilder};
use crate::container::stream::{Stream, StreamFactory};
use crate::container::ByteStream;
use crate::policy::{Reference, ReferenceFactory};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub struct ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    factory: F,
    capacity: usize,
    unused: PhantomData<(T, S)>,
}

impl<K, V, S, F> ByteStreamBuilder<(K, V), S, F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    pub fn forward<R, RB: Builder<R>>(
        self,
    ) -> ForwardBuilder<
        ByteStream<(K, V), S, F>,
        ByteStreamBuilder<(K, V), S, F>,
        R,
        RB,
    > {
        ForwardBuilder::new(self)
    }

    pub fn with_policy<R: Reference<V>, RF: ReferenceFactory<V, R>>(
        self,
        policy: RF,
    ) -> PolicyBuilder<
        ByteStream<(K, V), S, F>,
        V,
        R,
        RF,
        ByteStreamBuilder<(K, V), S, F>,
    > {
        PolicyBuilder::new(self, policy)
    }
}

impl<T, S, F> Builder<ByteStream<T, S, F>> for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    fn build(self) -> ByteStream<T, S, F> {
        ByteStream::new(self.factory, self.capacity)
    }
}

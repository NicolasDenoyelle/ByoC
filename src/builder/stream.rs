use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::container::stream::{Stream, StreamFactory};
use crate::container::ByteStream;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// [ByteStream](../../container/struct.ByteStream.html)
/// [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to spawn a
/// [ByteStream](../../container/struct.ByteStream.html) container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::container::stream::vec_stream::VecStreamFactory;
/// use cache::builder::builders::ByteStreamBuilder;
///
/// let mut stream = ByteStreamBuilder::new(VecStreamFactory{}, 2).build();
/// stream.push(vec![(1, 2)]);
/// ```
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

impl<T, S, F> ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    pub fn new(factory: F, capacity: usize) -> Self {
        ByteStreamBuilder {
            factory: factory,
            capacity: capacity,
            unused: PhantomData,
        }
    }
}

impl<T, S, F> Clone for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    fn clone(&self) -> Self {
        ByteStreamBuilder {
            factory: self.factory.clone(),
            capacity: self.capacity,
            unused: PhantomData,
        }
    }
}

impl<T, S, F> Associative<ByteStream<T, S, F>>
    for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Sequential<ByteStream<T, S, F>>
    for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Profiler<ByteStream<T, S, F>> for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F, R, RB> Multilevel<ByteStream<T, S, F>, R, RB>
    for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
    RB: Builder<R>,
{
}

impl<T, S, F> Policy<ByteStream<T, S, F>> for ByteStreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
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

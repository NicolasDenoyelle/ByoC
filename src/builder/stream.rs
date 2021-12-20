use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::streams::{Stream, StreamFactory};
use crate::Stream as ByteStream;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// `Stream` container builder.
///
/// This builder can be consumed later to spawn a
/// [`Stream`](../../struct.Stream.html) container.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::streams::VecStreamFactory;
/// use byoc::builder::builders::StreamBuilder;
///
/// let mut stream = StreamBuilder::new(VecStreamFactory{}, 2).build();
/// stream.push(vec![(1, 2)]);
/// ```
pub struct StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    factory: F,
    capacity: usize,
    unused: PhantomData<(T, S)>,
}

impl<T, S, F> StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    pub fn new(factory: F, capacity: usize) -> Self {
        StreamBuilder {
            factory,
            capacity,
            unused: PhantomData,
        }
    }
}

impl<T, S, F> Clone for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    fn clone(&self) -> Self {
        StreamBuilder {
            factory: self.factory.clone(),
            capacity: self.capacity,
            unused: PhantomData,
        }
    }
}

impl<T, S, F> Associative<ByteStream<T, S, F>> for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Sequential<ByteStream<T, S, F>> for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Profiler<ByteStream<T, S, F>> for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F, R, RB> Multilevel<ByteStream<T, S, F>, R, RB>
    for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
    RB: Builder<R>,
{
}

impl<T, S, F> Policy<ByteStream<T, S, F>> for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Builder<ByteStream<T, S, F>> for StreamBuilder<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
    fn build(self) -> ByteStream<T, S, F> {
        ByteStream::new(self.factory, self.capacity)
    }
}

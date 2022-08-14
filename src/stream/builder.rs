use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::stream::{Stream, StreamFactory};
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
/// use byoc::stream::VecStreamFactory;
/// use byoc::builder::builders::StreamBuilder;
///
/// let mut stream = StreamBuilder::new(VecStreamFactory{}, 2).build();
/// stream.push(vec![(1, 2)]);
/// ```
pub struct StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
    factory: F,
    capacity: usize,
    unused: PhantomData<&'a (T, S)>,
}

impl<'a, T, S, F> StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
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

impl<'a, T, S, F> Clone for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
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

impl<'a, T, S, F, H: std::hash::Hasher + Clone>
    Associative<ByteStream<'a, T, S, F>, H> for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
}

impl<'a, T, S, F> Sequential<ByteStream<'a, T, S, F>>
    for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
}

impl<'a, T, S, F> Profiler<ByteStream<'a, T, S, F>>
    for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
}

impl<'a, T, S, F, R, RB> Multilevel<ByteStream<'a, T, S, F>, R, RB>
    for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
    RB: Builder<R>,
{
}

impl<'a, T, S, F> Policy<ByteStream<'a, T, S, F>>
    for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
}

impl<'a, T, S, F> Builder<ByteStream<'a, T, S, F>>
    for StreamBuilder<'a, T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
    fn build(self) -> ByteStream<'a, T, S, F> {
        ByteStream::new(self.factory, self.capacity)
    }
}

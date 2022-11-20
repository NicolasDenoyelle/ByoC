use crate::builder::Build;
use crate::stream::StreamFactory;
use crate::Stream as ByteStream;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// `Stream` container builder.
///
/// This builder can be consumed later to spawn a
/// [`Stream`](../../struct.Stream.html) container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::utils::stream::VecStreamFactory;
/// use byoc::builder::StreamBuilder;
///
/// let mut stream = StreamBuilder::new(VecStreamFactory{}, 2).build();
/// stream.push(vec![(1, 2)]);
/// ```
pub struct StreamBuilder<T, F> {
    factory: F,
    pub(super) capacity: usize,
    unused: PhantomData<T>,
}

impl<T, F> StreamBuilder<T, F> {
    pub fn new(factory: F, capacity: usize) -> Self {
        StreamBuilder {
            factory,
            capacity,
            unused: PhantomData,
        }
    }
}

impl<T, F> Clone for StreamBuilder<T, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        StreamBuilder {
            factory: self.factory.clone(),
            capacity: self.capacity,
            unused: PhantomData,
        }
    }
}

impl<T, F> Build<ByteStream<T, F>> for StreamBuilder<T, F>
where
    T: DeserializeOwned + Serialize,
    F: StreamFactory + Clone,
{
    fn build(self) -> ByteStream<T, F> {
        ByteStream::new(self.factory, self.capacity)
    }
}

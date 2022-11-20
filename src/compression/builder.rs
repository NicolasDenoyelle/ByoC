use crate::builder::Build;
use crate::stream::StreamFactory;
use crate::Compressed;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for `Compression` building block.
///
/// If the [`StreamFactory`](../../utils/stream/trait.StreamFactory.html)
/// is `Clone`, then so is [`CompressedBuilder`]. However, it is desirable
/// that the cloned
/// [`StreamFactory`](../../utils/stream/trait.StreamFactory.html) does not
/// create the same streams because cloning this builder is likely used to build
/// an [`Associative`](../../struct.Associative.html) container.
pub struct CompressedBuilder<T, F> {
    pub(super) capacity: usize,
    stream_factory: F,
    unused: PhantomData<T>,
}

impl<T, F> CompressedBuilder<T, F> {
    pub fn new(capacity: usize, stream_factory: F) -> Self {
        CompressedBuilder {
            capacity,
            stream_factory,
            unused: PhantomData,
        }
    }
}

impl<T, F> Clone for CompressedBuilder<T, F>
where
    T: Serialize + DeserializeOwned,
    F: StreamFactory + Clone,
{
    fn clone(&self) -> Self {
        CompressedBuilder {
            capacity: self.capacity,
            stream_factory: self.stream_factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<T, F> Build<Compressed<T, F::Stream>> for CompressedBuilder<T, F>
where
    T: Serialize + DeserializeOwned,
    F: StreamFactory,
{
    fn build(mut self) -> Compressed<T, F::Stream> {
        Compressed::new(self.stream_factory.create(), self.capacity)
    }
}

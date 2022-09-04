use crate::builder::Build;
use crate::stream::StreamFactory;
use crate::{Batch, Compressed};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for `Compression` building block.
///
/// This builder will create a [`batch`](../../struct.Batch.html) of
/// `num_batch` smaller `Compression` building blocks, each with a set
/// `batch_capacity` on a its own stream created with `stream_factory`.
pub struct CompressedBuilder<T, F>
where
    T: Serialize + DeserializeOwned,
    F: StreamFactory,
{
    num_batch: usize,
    batch_capacity: usize,
    stream_factory: F,
    unused: PhantomData<T>,
}

impl<T, F> CompressedBuilder<T, F>
where
    T: Serialize + DeserializeOwned,
    F: StreamFactory,
{
    pub fn new(
        num_batch: usize,
        batch_capacity: usize,
        stream_factory: F,
    ) -> Self {
        CompressedBuilder {
            num_batch,
            batch_capacity,
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
            num_batch: self.num_batch,
            batch_capacity: self.batch_capacity,
            stream_factory: self.stream_factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<T, F> Build<Batch<Compressed<T, F::Stream>>>
    for CompressedBuilder<T, F>
where
    T: Serialize + DeserializeOwned,
    F: StreamFactory,
{
    fn build(mut self) -> Batch<Compressed<T, F::Stream>> {
        (0..self.num_batch).fold(
            Batch::<Compressed<T, F::Stream>>::new(),
            |acc, _| {
                acc.append(Compressed::new(
                    self.stream_factory.create(),
                    self.batch_capacity,
                ))
            },
        )
    }
}

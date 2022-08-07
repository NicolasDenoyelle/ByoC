use crate::builder::Build;
use crate::stream::{Stream, StreamFactory};
use crate::{Batch, Compressed};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for `Compression` building block.
///
/// This builder will create a [`batch`](../../struct.Batch.html) of
/// `num_batch` smaller `Compression` building blocks, each with a set
/// `batch_capacity` on a its own stream created with `stream_factory`.
pub struct CompressedBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
    num_batch: usize,
    batch_capacity: usize,
    stream_factory: F,
    unused: PhantomData<(T, S)>,
}

impl<T, S, F> CompressedBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
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

impl<T, S, F> Clone for CompressedBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S> + Clone,
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

impl<T, S, F> Build<Batch<Compressed<T, S>>> for CompressedBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
    fn build(mut self) -> Batch<Compressed<T, S>> {
        (0..self.num_batch).fold(
            Batch::<Compressed<T, S>>::new(),
            |acc, _| {
                acc.append(Compressed::new(
                    self.stream_factory.create(),
                    self.batch_capacity,
                ))
            },
        )
    }
}

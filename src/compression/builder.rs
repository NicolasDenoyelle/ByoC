use crate::builder::traits::*;
use crate::stream::{Stream, StreamFactory};
use crate::{Batch, Compressor};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for `Compression` building block.
///
/// This builder will create a [`batch`](../../struct.Batch.html) of
/// `num_batch` smaller `Compression` building blocks, each with a set
/// `batch_capacity` on a its own stream created with `stream_factory`.
pub struct CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
    num_batch: usize,
    batch_capacity: usize,
    stream_factory: F,
    unused: PhantomData<&'a (T, S)>,
}

impl<'a, T, S, F> CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
    pub fn new(
        num_batch: usize,
        batch_capacity: usize,
        stream_factory: F,
    ) -> Self {
        CompressorBuilder {
            num_batch,
            batch_capacity,
            stream_factory,
            unused: PhantomData,
        }
    }
}

impl<'a, T, S, F> Clone for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
    fn clone(&self) -> Self {
        CompressorBuilder {
            num_batch: self.num_batch,
            batch_capacity: self.batch_capacity,
            stream_factory: self.stream_factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, T, S, F> Builder<Batch<Compressor<'a, T, S>>>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
    fn build(mut self) -> Batch<Compressor<'a, T, S>> {
        let mut b = Batch::<Compressor<'a, T, S>>::new();
        for _ in 0..self.num_batch {
            b.append(Compressor::new(
                self.stream_factory.create(),
                self.batch_capacity,
            ));
        }
        b
    }
}

impl<'a, T, S, F, H: std::hash::Hasher + Clone>
    Associative<Batch<Compressor<'a, T, S>>, H>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S> + Clone,
{
}

impl<'a, T, S, F> Policy<Batch<Compressor<'a, T, S>>>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
}

impl<'a, T, S, F> Profiler<Batch<Compressor<'a, T, S>>>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
}

impl<'a, T, S, F> Sequential<Batch<Compressor<'a, T, S>>>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
{
}

impl<'a, T, S, F, R, RB> Multilevel<Batch<Compressor<'a, T, S>>, R, RB>
    for CompressorBuilder<'a, T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream<'a>,
    F: StreamFactory<S>,
    RB: Builder<R>,
{
}

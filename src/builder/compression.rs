use crate::builder::traits::*;
use crate::streams::{Stream, StreamFactory};
use crate::{Batch, CompressedContainer};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub struct CompressorBuilder<T, S, F>
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

impl<T, S, F> CompressorBuilder<T, S, F>
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
        CompressorBuilder {
            num_batch,
            batch_capacity,
            stream_factory,
            unused: PhantomData,
        }
    }
}

impl<T, S, F> Clone for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
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

impl<T, S, F> Builder<Batch<CompressedContainer<T, S>>>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
    fn build(mut self) -> Batch<CompressedContainer<T, S>> {
        let mut b = Batch::<CompressedContainer<T, S>>::new();
        for _ in 0..self.num_batch {
            b = b.append(CompressedContainer::new(
                self.stream_factory.create(),
                self.batch_capacity,
            ));
        }
        b
    }
}

impl<T, S, F> Associative<Batch<CompressedContainer<T, S>>>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S> + Clone,
{
}

impl<T, S, F> Policy<Batch<CompressedContainer<T, S>>>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
}

impl<T, S, F> Profiler<Batch<CompressedContainer<T, S>>>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
}

impl<T, S, F> Sequential<Batch<CompressedContainer<T, S>>>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
{
}

impl<T, S, F, R, RB> Multilevel<Batch<CompressedContainer<T, S>>, R, RB>
    for CompressorBuilder<T, S, F>
where
    T: Serialize + DeserializeOwned,
    S: Stream,
    F: StreamFactory<S>,
    RB: Builder<R>,
{
}

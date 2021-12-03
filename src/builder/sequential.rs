use crate::builder::Builder;
use crate::concurrent::Sequential;
use std::marker::PhantomData;

pub struct SequentialBuilder<C, B>
where
    B: Builder<C>,
{
    builder: B,
    unused: PhantomData<C>,
}

impl<C, B> Builder<Sequential<C>> for SequentialBuilder<C, B>
where
    B: Builder<C>,
{
    fn build(self) -> Sequential<C> {
        Sequential::new(self.builder.build())
    }
}

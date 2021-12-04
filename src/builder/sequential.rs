use crate::builder::traits::{Associative, Builder, Forward, Policy};
use crate::concurrent::Sequential;
use std::marker::PhantomData;

pub struct SequentialBuilder<C, B>
where
    B: Builder<C>,
{
    builder: B,
    unused: PhantomData<C>,
}

impl<C, B> SequentialBuilder<C, B>
where
    B: Builder<C>,
{
    pub fn new(builder: B) -> Self {
        SequentialBuilder {
            builder: builder,
            unused: PhantomData,
        }
    }
}

impl<C, B> Clone for SequentialBuilder<C, B>
where
    B: Builder<C> + Clone,
{
    fn clone(&self) -> Self {
        SequentialBuilder {
            builder: self.builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, B> Associative<Sequential<C>> for SequentialBuilder<C, B> where
    B: Builder<C> + Clone
{
}

impl<C, B> Policy<Sequential<C>> for SequentialBuilder<C, B> where
    B: Builder<C>
{
}

impl<L, LB, R, RB> Forward<Sequential<L>, R, RB>
    for SequentialBuilder<L, LB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<C, B> Builder<Sequential<C>> for SequentialBuilder<C, B>
where
    B: Builder<C>,
{
    fn build(self) -> Sequential<C> {
        Sequential::new(self.builder.build())
    }
}

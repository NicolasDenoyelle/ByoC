use crate::builder::traits::{Associative, Builder, Forward, Policy, Profiler};
use crate::concurrent::Sequential;
use std::marker::PhantomData;

/// [Sequential](../../concurrent/struct.Sequential.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to wrap some containers into a
/// [Sequential](../../concurrent/struct.Sequential.html)
/// container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::{ArrayBuilder, SequentialBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = SequentialBuilder::new(array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).into_sequential().build();
/// container.push(vec![(1, 2)]);
/// ```
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

impl<C, B> Profiler<Sequential<C>> for SequentialBuilder<C, B> where
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

use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler,
};
use crate::Sequential;
use std::marker::PhantomData;

/// `Sequential` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Sequential`](../../struct.Sequential.html)
/// container.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::builder::builders::{ArrayBuilder, SequentialBuilder};
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
            builder,
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

impl<C, B, H: std::hash::Hasher + Clone> Associative<Sequential<C>, H>
    for SequentialBuilder<C, B>
where
    B: Builder<C> + Clone,
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

impl<L, LB, R, RB> Multilevel<Sequential<L>, R, RB>
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

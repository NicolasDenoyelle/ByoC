use crate::builder::traits::{Associative, Builder, Multilevel};
use crate::profiler::Profiler;
use std::marker::PhantomData;

/// [Profiler](../../profiler/struct.Profiler.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to wrap some containers into a
/// [Profiler](../../profiler/struct.Profiler.html)
/// container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::{ArrayBuilder, ProfilerBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = ProfilerBuilder::new(array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).profile().build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ProfilerBuilder<C, B>
where
    B: Builder<C>,
{
    builder: B,
    unused: PhantomData<C>,
}

impl<C, B> ProfilerBuilder<C, B>
where
    B: Builder<C>,
{
    pub fn new(builder: B) -> Self {
        ProfilerBuilder {
            builder: builder,
            unused: PhantomData,
        }
    }
}

impl<C, B> Clone for ProfilerBuilder<C, B>
where
    B: Builder<C> + Clone,
{
    fn clone(&self) -> Self {
        ProfilerBuilder {
            builder: self.builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, B> Associative<Profiler<C>> for ProfilerBuilder<C, B> where
    B: Builder<C> + Clone
{
}

impl<L, LB, R, RB> Multilevel<Profiler<L>, R, RB>
    for ProfilerBuilder<L, LB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<C, B> Builder<Profiler<C>> for ProfilerBuilder<C, B>
where
    B: Builder<C>,
{
    fn build(self) -> Profiler<C> {
        Profiler::new(self.builder.build())
    }
}

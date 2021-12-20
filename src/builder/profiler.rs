use crate::builder::traits::{Associative, Builder, Multilevel};
use crate::{Profiler, ProfilerOutputKind};
use std::marker::PhantomData;

/// `Profiler` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Profiler`](../../struct.Profiler.html)
/// container.
///
/// # Examples
///
/// ```
/// use byoc::{BuildingBlock, ProfilerOutputKind};
/// use byoc::builder::traits::*;
/// use byoc::builder::builders::{ArrayBuilder, ProfilerBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = ProfilerBuilder::new("example", ProfilerOutputKind::None, array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).profile("example_builder", ProfilerOutputKind::None).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ProfilerBuilder<C, B>
where
    B: Builder<C>,
{
    builder: B,
    name: String,
    output: ProfilerOutputKind,
    unused: PhantomData<C>,
}

impl<C, B> ProfilerBuilder<C, B>
where
    B: Builder<C>,
{
    pub fn new(
        name: &str,
        output: ProfilerOutputKind,
        builder: B,
    ) -> Self {
        ProfilerBuilder {
            builder,
            name: String::from(name),
            output,
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
            name: self.name.clone(),
            output: self.output.clone(),
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
        Profiler::new(
            self.name.as_ref(),
            self.output,
            self.builder.build(),
        )
    }
}

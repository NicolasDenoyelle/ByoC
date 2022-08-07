use crate::builder::Build;
use crate::utils::profiler::ProfilerOutputKind;
use crate::Profiler;
use std::marker::PhantomData;

/// `Profiler` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Profiler`](../../struct.Profiler.html)
/// container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::utils::profiler::ProfilerOutputKind;
/// use byoc::builder::Build;
/// use byoc::builder::builders::{ArrayBuilder, ProfilerBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container =
///     ProfilerBuilder::new("example",
///                          ProfilerOutputKind::None,
///                          array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2)
///     .profile("example_builder", ProfilerOutputKind::None)
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ProfilerBuilder<C, B>
where
    B: Build<C>,
{
    builder: B,
    name: String,
    output: ProfilerOutputKind,
    unused: PhantomData<C>,
}

impl<C, B> ProfilerBuilder<C, B>
where
    B: Build<C>,
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
    B: Build<C> + Clone,
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

impl<C, B> Build<Profiler<C>> for ProfilerBuilder<C, B>
where
    B: Build<C>,
{
    fn build(self) -> Profiler<C> {
        Profiler::new(
            self.name.as_ref(),
            self.output,
            self.builder.build(),
        )
    }
}

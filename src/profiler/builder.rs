use crate::builder::Build;
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
/// use byoc::builder::{Build,ProfilerBuild};
/// use byoc::builder::{ArrayBuilder, ProfilerBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container =
///     ProfilerBuilder::new(array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2)
///     .profile()
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ProfilerBuilder<C, B> {
    pub(super) builder: B,
    unused: PhantomData<C>,
}

impl<C, B> ProfilerBuilder<C, B> {
    pub fn new(builder: B) -> Self {
        ProfilerBuilder {
            builder,
            unused: PhantomData,
        }
    }
}

impl<C, B> Clone for ProfilerBuilder<C, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        ProfilerBuilder {
            builder: self.builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, B> Build<Profiler<C>> for ProfilerBuilder<C, B>
where
    B: Build<C>,
{
    fn build(self) -> Profiler<C> {
        Profiler::new(self.builder.build())
    }
}

/// Add profiling to a container [`Build`].
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,ProfilerBuild};
///
/// let mut container = Builder::array(10000)
///                    .profile()
///                    .build();
/// container.push(vec![(1,2)]);
/// ```
pub trait ProfilerBuild<C>: Build<C> {
    /// [Profile](../../struct.Profiler.html) the preceding
    /// building block.
    ///
    /// The output profile will be identified by its `name` and will
    /// be available in `output` once the container is dropped.
    fn profile(self) -> ProfilerBuilder<C, Self>
    where
        Self: Sized,
    {
        ProfilerBuilder::new(self)
    }
}

impl<C, B: Build<C>> ProfilerBuild<C> for B {}

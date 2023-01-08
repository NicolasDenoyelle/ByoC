use super::FlushStopper;
use crate::builder::Build;
use std::marker::PhantomData;

/// `FlushStopper` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`FlushStopper`](../../struct.FlushStopper.html) container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build, FlushStopperBuild};
/// use byoc::builder::{ArrayBuilder, FlushStopperBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = FlushStopperBuilder::new(array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).dont_flush().build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct FlushStopperBuilder<C, B> {
    pub(super) builder: B,
    unused: PhantomData<C>,
}

impl<C, B> FlushStopperBuilder<C, B> {
    pub fn new(builder: B) -> Self {
        FlushStopperBuilder {
            builder,
            unused: PhantomData,
        }
    }
}

impl<C, B> Clone for FlushStopperBuilder<C, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        FlushStopperBuilder {
            builder: self.builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, B> Build<FlushStopper<C>> for FlushStopperBuilder<C, B>
where
    B: Build<C>,
{
    fn build(self) -> FlushStopper<C> {
        FlushStopper::new(self.builder.build())
    }
}

/// Make the container being unable to flush its elements.
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,FlushStopperBuild};
///
/// let mut container = Builder::array(10000).dont_flush().build();
/// container.push(vec![(1,2)]);
/// ```
pub trait FlushStopperBuild<C> {
    /// Wrap a container builder into a
    /// [flush stopper](../../struct.FlushStopper.html) building block
    /// to secure prevent the container from being able to flush its elements.
    fn dont_flush(self) -> FlushStopperBuilder<C, Self>
    where
        Self: Sized,
    {
        FlushStopperBuilder::new(self)
    }
}

impl<C, B: Build<C>> FlushStopperBuild<C> for B {}

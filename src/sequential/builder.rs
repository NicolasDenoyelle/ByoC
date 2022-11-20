use crate::builder::Build;
use crate::Sequential;
use std::marker::PhantomData;

/// `Sequential` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Sequential`](../../struct.Sequential.html)
/// container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build, SequentialBuild};
/// use byoc::builder::{ArrayBuilder, SequentialBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = SequentialBuilder::new(array_builder).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).into_sequential().build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct SequentialBuilder<C, B> {
    pub(super) builder: B,
    unused: PhantomData<C>,
}

impl<C, B> SequentialBuilder<C, B> {
    pub fn new(builder: B) -> Self {
        SequentialBuilder {
            builder,
            unused: PhantomData,
        }
    }
}

impl<C, B> Clone for SequentialBuilder<C, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        SequentialBuilder {
            builder: self.builder.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, B> Build<Sequential<C>> for SequentialBuilder<C, B>
where
    B: Build<C>,
{
    fn build(self) -> Sequential<C> {
        Sequential::new(self.builder.build())
    }
}

/// Make the container being built thread safe by sequentializing its accesses.
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,SequentialBuild};
///
/// let mut container = Builder::array(10000).into_sequential().build();
/// container.push(vec![(1,2)]);
/// ```
pub trait SequentialBuild<C> {
    /// Wrap a container builder into a
    /// [sequential](../../struct.Sequential.html) building block
    /// to secure concurrent access behind a lock.
    fn into_sequential(self) -> SequentialBuilder<C, Self>
    where
        Self: Sized,
    {
        SequentialBuilder::new(self)
    }
}

impl<C, B: Build<C>> SequentialBuild<C> for B {}

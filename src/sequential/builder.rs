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
/// use byoc::builder::Build;
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
    B: Build<C>,
{
    builder: B,
    unused: PhantomData<C>,
}

impl<C, B> SequentialBuilder<C, B>
where
    B: Build<C>,
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
    B: Build<C> + Clone,
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

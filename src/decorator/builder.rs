use crate::builder::Build;
use crate::decorator::DecorationFactory;
use crate::Decorator;
use std::marker::PhantomData;

/// `Decorator` container builder.
///
/// This builder will wrap the values of the built container into a
/// [`Decoration`](../../decorator/struct.Decorator.html)
/// [`Decorator`](../../struct.Decorator.html)
/// container, thus applying an eviction policy to the wrapped container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,DecoratorBuild};
/// use byoc::utils::decorator::Fifo;
/// use byoc::builder::{ArrayBuilder, DecoratorBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container =
///     DecoratorBuilder::new(array_builder, Fifo::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container =
///    ArrayBuilder::new(2).with_decorator(Fifo::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct DecoratorBuilder<C, V, F, B> {
    pub(super) builder: B,
    pub(super) decorator: F,
    unused: PhantomData<(C, V)>,
}

impl<C, V, F, B> Clone for DecoratorBuilder<C, V, F, B>
where
    B: Build<C> + Clone,
    F: DecorationFactory<V> + Clone,
{
    fn clone(&self) -> Self {
        DecoratorBuilder {
            builder: self.builder.clone(),
            decorator: self.decorator.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, V, F, B> DecoratorBuilder<C, V, F, B> {
    pub fn new(builder: B, decorator: F) -> Self {
        Self {
            builder,
            decorator,
            unused: PhantomData,
        }
    }
}

impl<C, V, F, B> Build<Decorator<C, V, F>> for DecoratorBuilder<C, V, F, B>
where
    B: Build<C>,
    F: DecorationFactory<V>,
{
    fn build(self) -> Decorator<C, V, F> {
        Decorator::new(self.builder.build(), self.decorator)
    }
}

/// Attach a `Decorator` to a container [`Build`].
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,DecoratorBuild};
/// use byoc::utils::decorator::Fifo;
///
/// let mut container = Builder::array(10000)
///                    .with_decorator(Fifo::new())
///                    .build();
/// container.push(vec![(1,2)]);
/// ```
pub trait DecoratorBuild<C>: Build<C> {
    /// [`Decorator`](../../struct.Decorator.html)
    /// wrapping capability.
    fn with_decorator<V, F: DecorationFactory<V>>(
        self,
        decorator: F,
    ) -> DecoratorBuilder<C, V, F, Self>
    where
        Self: Sized,
    {
        DecoratorBuilder::new(self, decorator)
    }
}

impl<C, B: Build<C>> DecoratorBuild<C> for B {}

use crate::builder::Build;
use crate::policy::{Ordered, ReferenceFactory};
use crate::Policy;
use std::marker::PhantomData;

/// `Policy` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Policy`](../../struct.Policy.html)
/// container, thus applying an eviction policy to the wrapped container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,PolicyBuild};
/// use byoc::policy::Fifo;
/// use byoc::builder::{ArrayBuilder, PolicyBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container =
///     PolicyBuilder::new(array_builder, Fifo::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container =
///    ArrayBuilder::new(2).with_policy(Fifo::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct PolicyBuilder<C, V, F, B>
where
    C: Ordered<F::Item>,
    B: Build<C>,
    F: ReferenceFactory<V>,
{
    builder: B,
    policy: F,
    unused: PhantomData<(C, V)>,
}

impl<C, V, F, B> Clone for PolicyBuilder<C, V, F, B>
where
    C: Ordered<F::Item>,
    B: Build<C> + Clone,
    F: ReferenceFactory<V> + Clone,
{
    fn clone(&self) -> Self {
        PolicyBuilder {
            builder: self.builder.clone(),
            policy: self.policy.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, V, F, B> PolicyBuilder<C, V, F, B>
where
    C: Ordered<F::Item>,
    B: Build<C>,
    F: ReferenceFactory<V>,
{
    pub fn new(builder: B, policy: F) -> Self {
        Self {
            builder,
            policy,
            unused: PhantomData,
        }
    }
}

impl<C, V, F, B> Build<Policy<C, V, F>> for PolicyBuilder<C, V, F, B>
where
    C: Ordered<F::Item>,
    B: Build<C>,
    F: ReferenceFactory<V>,
{
    fn build(self) -> Policy<C, V, F> {
        Policy::new(self.builder.build(), self.policy)
    }
}

/// Attach an ordering policy to a container [`Build`].
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,PolicyBuild};
/// use byoc::policy::Fifo;
///
/// let mut container = Builder::array(10000)
///                    .with_policy(Fifo::new())
///                    .build();
/// container.push(vec![(1,2)]);
/// ```
pub trait PolicyBuild<C>: Build<C> {
    /// [`Policy`](../../struct.Policy.html)
    /// wrapping capability.
    fn with_policy<V, F: ReferenceFactory<V>>(
        self,
        policy: F,
    ) -> PolicyBuilder<C, V, F, Self>
    where
        Self: Sized,
        C: Ordered<F::Item>,
    {
        PolicyBuilder::new(self, policy)
    }
}

impl<C, B: Build<C>> PolicyBuild<C> for B {}

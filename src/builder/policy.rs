use crate::builder::traits::{Associative, Builder, Multilevel, Sequential};
use crate::policy::{Policy, Reference, ReferenceFactory};
use std::marker::PhantomData;

/// [Policy](../../policy/policy/struct.Policy.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to wrap some containers into a
/// [Policy](../../container/concurrent/struct.Associative.html)
/// container, thus applying an eviction policy to the wrapped container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::policy::FIFO;
/// use cache::builder::builders::{ArrayBuilder, PolicyBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = PolicyBuilder::new(array_builder, FIFO::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).with_policy(FIFO::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C>,
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    builder: B,
    policy: F,
    unused: PhantomData<(C, V, R)>,
}

impl<C, V, R, F, B> Clone for PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C> + Clone,
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
{
    fn clone(&self) -> Self {
        PolicyBuilder {
            builder: self.builder.clone(),
            policy: self.policy.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, V, R, F, B> Associative<Policy<C, V, R, F>>
    for PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C> + Clone,
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
{
}

impl<C, V, R, F, B> Sequential<Policy<C, V, R, F>>
    for PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C> + Clone,
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
{
}

impl<C, V, R, F, B> PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C>,
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    pub fn new(builder: B, policy: F) -> Self {
        Self {
            builder: builder,
            policy: policy,
            unused: PhantomData,
        }
    }
}

impl<L, V, I, F, LB, R, RB> Multilevel<Policy<L, V, I, F>, R, RB>
    for PolicyBuilder<L, V, I, F, LB>
where
    LB: Builder<L>,
    I: Reference<V>,
    F: ReferenceFactory<V, I>,
    RB: Builder<R>,
{
}

impl<C, V, R, F, B> Builder<Policy<C, V, R, F>>
    for PolicyBuilder<C, V, R, F, B>
where
    B: Builder<C>,
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    fn build(self) -> Policy<C, V, R, F> {
        Policy::new(self.builder.build(), self.policy)
    }
}

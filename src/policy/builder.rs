use crate::builder::traits::{
    Associative, Builder, Multilevel, Sequential,
};
use crate::policy::{Reference, ReferenceFactory};
use crate::Policy;
use std::marker::PhantomData;

/// `Policy` container builder.
///
/// This builder can be consumed later to wrap some containers into a
/// [`Policy`](../../struct.Policy.html)
/// container, thus applying an eviction policy to the wrapped container.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::policy::FIFO;
/// use byoc::builder::builders::{ArrayBuilder, PolicyBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container =
///     PolicyBuilder::new(array_builder, FIFO::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container =
///    ArrayBuilder::new(2).with_policy(FIFO::new()).build();
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

impl<C, V, R, F, B, H: std::hash::Hasher + Clone>
    Associative<Policy<C, V, R, F>, H> for PolicyBuilder<C, V, R, F, B>
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
            builder,
            policy,
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

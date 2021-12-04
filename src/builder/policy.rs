use crate::builder::traits::{Associative, Builder, Forward, Sequential};
use crate::policy::{Policy, Reference, ReferenceFactory};
use std::marker::PhantomData;

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

impl<L, V, I, F, LB, R, RB> Forward<Policy<L, V, I, F>, R, RB>
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

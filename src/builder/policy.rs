use crate::builder::Builder;
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

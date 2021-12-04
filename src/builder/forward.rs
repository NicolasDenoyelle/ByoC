use crate::builder::builders::{
    AssociativeBuilder, PolicyBuilder, SequentialBuilder,
};
use crate::builder::traits::{
    Associative, Builder, Forward as ForwardTo, Policy, Sequential,
};

use crate::connector::Forward;
use crate::policy::{Reference, ReferenceFactory};
use std::hash::Hasher;
use std::marker::PhantomData;

/// [Forward](../../connector/struct.Forward.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to connect two containers together
/// with a [Forward](../../connector/struct.Forward.html) connector.
/// It is created from two other builders that will build the left hand
/// side of the connection and the right hand side of the connection.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::{ArrayBuilder, BTreeBuilder, ForwardBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let btree_builder = BTreeBuilder::new(2);
/// let forward_builder = ForwardBuilder::new(array_builder, btree_builder);
/// let mut container = forward_builder.build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls.
/// let mut container = ArrayBuilder::new(2).forward(BTreeBuilder::new(2)).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    lbuilder: LB,
    rbuilder: RB,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> Clone for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
    fn clone(&self) -> Self {
        ForwardBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: self.rbuilder.clone(),
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    pub fn new(lbuilder: LB, rbuilder: RB) -> Self {
        Self {
            lbuilder: lbuilder,
            rbuilder: rbuilder,
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> Sequential<Forward<L, R>>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn into_sequential(self) -> SequentialBuilder<Forward<L, R>, Self> {
        SequentialBuilder::new(self)
    }
}

impl<L, LB, R, RB> Associative<Forward<L, R>>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
    fn into_associative<H: Hasher + Clone>(
        self,
        n_sets: usize,
        key_hasher: H,
    ) -> AssociativeBuilder<Forward<L, R>, H, Self> {
        AssociativeBuilder::new(self, n_sets, key_hasher)
    }
}

impl<L, LB, R, RB> ForwardTo<Forward<L, R>, R, RB>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn forward(
        self,
        rbuilder: RB,
    ) -> ForwardBuilder<Forward<L, R>, Self, R, RB> {
        ForwardBuilder::new(self, rbuilder)
    }
}

impl<L, LB, R, RB> Policy<Forward<L, R>> for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn with_policy<V, I: Reference<V>, F: ReferenceFactory<V, I>>(
        self,
        policy: F,
    ) -> PolicyBuilder<Forward<L, R>, V, I, F, Self>
    where
        Self: Sized,
    {
        PolicyBuilder::new(self, policy)
    }
}

impl<L, LB, R, RB> Builder<Forward<L, R>> for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn build(self) -> Forward<L, R> {
        Forward::new(self.lbuilder.build(), self.rbuilder.build())
    }
}

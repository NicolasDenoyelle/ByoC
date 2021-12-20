use crate::builder::traits::{
    Associative, Builder, Multilevel as MultilevelTo, Policy, Profiler,
    Sequential,
};

use crate::Multilevel;
use std::marker::PhantomData;

/// `Multilevel` container builder.
///
/// This builder can be consumed later to connect two containers together
/// with a [`Multilevel`](../../struct.Multilevel.html) connector.
/// It is created from two other builders that will build the left hand
/// side of the connection and the right hand side of the connection.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::builder::builders::{ArrayBuilder, BTreeBuilder, MultilevelBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let btree_builder = BTreeBuilder::new(2);
/// let multilevel_builder = MultilevelBuilder::new(array_builder, btree_builder);
/// let mut container = multilevel_builder.build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls.
/// let mut container = ArrayBuilder::new(2).multilevel(BTreeBuilder::new(2)).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    lbuilder: LB,
    rbuilder: RB,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> Clone for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
    fn clone(&self) -> Self {
        MultilevelBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: self.rbuilder.clone(),
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    pub fn new(lbuilder: LB, rbuilder: RB) -> Self {
        Self {
            lbuilder,
            rbuilder,
            unused: PhantomData,
        }
    }
}

impl<K, V, L, LB, R, RB> Sequential<Multilevel<K, V, L, R>>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<K, V, L, LB, R, RB> Profiler<Multilevel<K, V, L, R>>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<K, V, L, LB, R, RB> Associative<Multilevel<K, V, L, R>>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
}

impl<K, V, L, LB, R, RB> MultilevelTo<Multilevel<K, V, L, R>, R, RB>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<K, V, L, LB, R, RB> Policy<Multilevel<K, V, L, R>>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
}

impl<K, V, L, LB, R, RB> Builder<Multilevel<K, V, L, R>>
    for MultilevelBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn build(self) -> Multilevel<K, V, L, R> {
        Multilevel::new(self.lbuilder.build(), self.rbuilder.build())
    }
}

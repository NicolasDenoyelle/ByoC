use crate::builder::Build;
use crate::Exclusive;
use std::marker::PhantomData;

/// `Exclusive` container builder.
///
/// This builder can be consumed later to connect two containers together
/// with a [`Exclusive`](../../struct.Exclusive.html) connector.
/// It is created from two other builders that will build the front hand
/// side of the connection and the back hand side of the connection.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::builder::builders::{
///     ArrayBuilder, BTreeBuilder, ExclusiveBuilder
/// };
///
/// let array_builder = ArrayBuilder::new(2);
/// let btree_builder = BTreeBuilder::new(2);
/// let exclusive_builder =
///     ExclusiveBuilder::new(array_builder, btree_builder);
/// let mut container = exclusive_builder.build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls.
/// let mut container = ArrayBuilder::new(2)
///     .exclusive(BTreeBuilder::new(2))
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ExclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L>,
    RB: Build<R>,
{
    lbuilder: LB,
    rbuilder: RB,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> Clone for ExclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L> + Clone,
    RB: Build<R> + Clone,
{
    fn clone(&self) -> Self {
        ExclusiveBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: self.rbuilder.clone(),
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> ExclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L>,
    RB: Build<R>,
{
    pub fn new(lbuilder: LB, rbuilder: RB) -> Self {
        Self {
            lbuilder,
            rbuilder,
            unused: PhantomData,
        }
    }
}

impl<K, V, L, LB, R, RB> Build<Exclusive<K, V, L, R>>
    for ExclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L>,
    RB: Build<R>,
{
    fn build(self) -> Exclusive<K, V, L, R> {
        Exclusive::new(self.lbuilder.build(), self.rbuilder.build())
    }
}

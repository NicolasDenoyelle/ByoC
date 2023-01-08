use super::{Inclusive, InclusiveCell};
use crate::builder::Build;
use crate::BuildingBlock;
use std::marker::PhantomData;

/// `Inclusive` container builder.
///
/// This builder can be consumed later to connect two containers together
/// with a [`Inclusive`](../../struct.Inclusive.html) connector.
/// It is created from two other builders that will build the front
/// side of the connection and the back side of the connection.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,InclusiveBuild};
/// use byoc::builder::{
///     ArrayBuilder, BTreeBuilder, InclusiveBuilder
/// };
///
/// let array_builder = ArrayBuilder::new(2);
/// let btree_builder = BTreeBuilder::new(2);
/// let inclusive_builder =
///     InclusiveBuilder::new(array_builder, btree_builder);
/// let mut container = inclusive_builder.build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls.
/// let mut container = ArrayBuilder::new(2)
///     .inclusive(BTreeBuilder::new(2))
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct InclusiveBuilder<L, LB, R, RB> {
    pub(super) lbuilder: LB,
    pub(super) rbuilder: RB,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> Clone for InclusiveBuilder<L, LB, R, RB>
where
    LB: Clone,
    RB: Clone,
{
    fn clone(&self) -> Self {
        InclusiveBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: self.rbuilder.clone(),
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> InclusiveBuilder<L, LB, R, RB> {
    pub fn new(lbuilder: LB, rbuilder: RB) -> Self {
        Self {
            lbuilder,
            rbuilder,
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, L, LB, R, RB> Build<Inclusive<'a, K, V, L, R>>
    for InclusiveBuilder<L, LB, R, RB>
where
    K: 'a + Clone,
    V: 'a + Clone,
    L: BuildingBlock<'a, K, InclusiveCell<V>>,
    R: BuildingBlock<'a, K, InclusiveCell<V>>,
    LB: Build<L>,
    RB: Build<R>,
{
    fn build(self) -> Inclusive<'a, K, V, L, R> {
        Inclusive::new(self.lbuilder.build(), self.rbuilder.build())
    }
}

/// Connect two containers [`Build`] into an `Inclusive` container.
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::{Build,Builder,InclusiveBuild};
///
/// let front = Builder::array(10000);
/// let back = Builder::array(10000);
/// let mut container = front.inclusive(back).build();
/// container.push(vec![(1,2)]);
/// ```
pub trait InclusiveBuild<C> {
    /// Connection between two building blocks with a
    /// [`Inclusive`](../../struct.Inclusive.html)
    /// [building block](../../trait.BuildingBlock.html).
    fn inclusive<R, RB: Build<R>>(
        self,
        rbuilder: RB,
    ) -> InclusiveBuilder<C, Self, R, RB>
    where
        Self: Sized,
    {
        InclusiveBuilder::new(self, rbuilder)
    }
}

impl<C, B: Build<C>> InclusiveBuild<C> for B {}

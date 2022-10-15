use super::{Inclusive, InclusiveCell};
use crate::builder::Build;
use crate::BuildingBlock;
use std::marker::PhantomData;

pub struct InclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L>,
    RB: Build<R>,
{
    lbuilder: LB,
    rbuilder: RB,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> Clone for InclusiveBuilder<L, LB, R, RB>
where
    LB: Build<L> + Clone,
    RB: Build<R> + Clone,
{
    fn clone(&self) -> Self {
        InclusiveBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: self.rbuilder.clone(),
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> InclusiveBuilder<L, LB, R, RB>
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

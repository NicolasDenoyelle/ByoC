use crate::builder::Builder;
use crate::connector::Forward;
use std::marker::PhantomData;

pub struct ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    lbuilder: LB,
    rbuilder: Option<RB>,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, R, RB> ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    pub fn new(lbuilder: LB) -> Self {
        Self {
            lbuilder: lbuilder,
            rbuilder: None,
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> Builder<Forward<L, R>> for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn build(self) -> Forward<L, R> {
        Forward::new(self.lbuilder.build(), self.rbuilder.expect("ForwardBuilder requires to connect a container on the right hand side.").build())
    }
}

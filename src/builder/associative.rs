use crate::builder::Builder;
use crate::concurrent::Associative;
use std::marker::PhantomData;
use std::hash::Hasher;

pub struct AssociativeBuilder<C, H, B>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    builder: B,
    n_sets: usize,
    set_hasher: H,
    unused: PhantomData<C>,
}

impl<C, H, B> Builder<Associative<C, H>> for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
    fn build(self) -> Associative<C, H> {
        let mut containers = Vec::new();
        for _ in 0..self.n_sets {
            containers.push(self.builder.clone().build());
        }
        Associative::new(containers, self.set_hasher)
    }
}

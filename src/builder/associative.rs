use crate::builder::traits::{Builder, Forward, Policy};
use crate::concurrent::Associative;
use std::hash::Hasher;
use std::marker::PhantomData;

/// [Associative](../../concurrent/struct.Associative.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to wrap some containers into an
/// [Associative](../../concurrent/struct.Associative.html)
/// container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::{ArrayBuilder, AssociativeBuilder};
/// use std::collections::hash_map::DefaultHasher;
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container = AssociativeBuilder::new(array_builder, 2, DefaultHasher::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).into_associative(2, DefaultHasher::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
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

impl<C, H, B> Clone for AssociativeBuilder<C, H, B>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    fn clone(&self) -> Self {
        AssociativeBuilder {
            builder: self.builder.clone(),
            n_sets: self.n_sets,
            set_hasher: self.set_hasher.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, H, B> AssociativeBuilder<C, H, B>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    pub fn new(builder: B, n_sets: usize, key_hasher: H) -> Self {
        AssociativeBuilder {
            builder: builder,
            n_sets: n_sets,
            set_hasher: key_hasher,
            unused: PhantomData,
        }
    }
}

impl<C, H, B> Policy<Associative<C, H>> for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<L, H, LB, R, RB> Forward<Associative<L, H>, R, RB>
    for AssociativeBuilder<L, H, LB>
where
    LB: Builder<L> + Clone,
    H: Hasher + Clone,
    RB: Builder<R>,
{
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

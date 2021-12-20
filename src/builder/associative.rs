use crate::builder::traits::{Builder, Multilevel, Policy, Profiler};
use crate::Associative;
use std::hash::Hasher;
use std::marker::PhantomData;

/// `Associative` container builder.
///
/// This builder can be consumed later to wrap some containers into an
/// [`Associative`](../../struct.Associative.html) container.
///
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::*;
/// use byoc::builder::builders::{ArrayBuilder, AssociativeBuilder};
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
            builder,
            n_sets,
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

impl<C, H, B> Profiler<Associative<C, H>> for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<L, H, LB, R, RB> Multilevel<Associative<L, H>, R, RB>
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

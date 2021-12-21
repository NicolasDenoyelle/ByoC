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
/// let mut container = AssociativeBuilder::<_,_,_,2>::new(array_builder, DefaultHasher::new()).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2).into_associative::<2>(DefaultHasher::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct AssociativeBuilder<C, H, B, const N: usize>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    builder: B,
    set_hasher: H,
    unused: PhantomData<C>,
}

impl<C, H, B, const N: usize> Clone for AssociativeBuilder<C, H, B, N>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    fn clone(&self) -> Self {
        AssociativeBuilder {
            builder: self.builder.clone(),
            set_hasher: self.set_hasher.clone(),
            unused: PhantomData,
        }
    }
}

impl<C, H, B, const N: usize> AssociativeBuilder<C, H, B, N>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    pub fn new(builder: B, key_hasher: H) -> Self {
        AssociativeBuilder {
            builder,
            set_hasher: key_hasher,
            unused: PhantomData,
        }
    }
}

impl<C, H, B, const N: usize> Policy<Associative<C, H, N>>
    for AssociativeBuilder<C, H, B, N>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<C, H, B, const N: usize> Profiler<Associative<C, H, N>>
    for AssociativeBuilder<C, H, B, N>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<L, H, LB, R, RB, const N: usize>
    Multilevel<Associative<L, H, N>, R, RB>
    for AssociativeBuilder<L, H, LB, N>
where
    LB: Builder<L> + Clone,
    H: Hasher + Clone,
    RB: Builder<R>,
{
}

macro_rules! array {
    ($x: expr, $n: ident) => {
        [(); $n].map(|_| $x)
    };
}

impl<C, H, B, const N: usize> Builder<Associative<C, H, N>>
    for AssociativeBuilder<C, H, B, N>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
    fn build(self) -> Associative<C, H, N> {
        Associative::new(
            array![self.builder.clone().build(), N],
            self.set_hasher,
        )
    }
}

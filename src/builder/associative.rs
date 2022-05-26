use crate::builder::traits::{Builder, Multilevel, Policy, Profiler};
use crate::{Associative, MultisetHasher};
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
/// let mut container =
///     AssociativeBuilder::<_,_,_>::new(array_builder,
///                                      DefaultHasher::new(), 2).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls:
/// let mut container = ArrayBuilder::new(2)
///     .into_associative(DefaultHasher::new(), 2)
///     .build();
/// container.push(vec![(1, 2)]);
///
/// // You can also build a multilayer associative container:
/// // Here with up to 8 elements capacity.
/// // At the bottom, elements are stored in 4 array containers.
/// // Array containers are further grouped into 2 groups of 2 arrays of
/// // 2 elements.
/// // When matching a key with an array container, the key is hashed once
/// // to find the group of array containers and a second time to find the
/// // right array container.
/// let mut container = ArrayBuilder::new(2)
///     .into_associative(DefaultHasher::new(), 2)
///     .add_layer(2)
///     .build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct AssociativeBuilder<C, H, B>
where
    H: Hasher + Clone,
    B: Builder<C> + Clone,
{
    builder: B,
    set_hasher: MultisetHasher<H>,
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
    pub fn new(builder: B, key_hasher: H, num_sets: usize) -> Self {
        AssociativeBuilder {
            builder,
            set_hasher: MultisetHasher::new(key_hasher, num_sets),
            unused: PhantomData,
        }
    }

    pub fn add_layer(
        self,
        num_keys: usize,
    ) -> AssociativeBuilder<Associative<C, MultisetHasher<H>>, H, Self>
    {
        let hasher = self.set_hasher.next(num_keys).expect(
            "Too many and/or too large associative layers stacked.",
        );
        AssociativeBuilder {
            builder: self,
            set_hasher: hasher,
            unused: PhantomData,
        }
    }
}

impl<C, H, B> Policy<Associative<C, MultisetHasher<H>>>
    for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<C, H, B> Profiler<Associative<C, MultisetHasher<H>>>
    for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
}

impl<L, H, LB, R, RB> Multilevel<Associative<L, MultisetHasher<H>>, R, RB>
    for AssociativeBuilder<L, H, LB>
where
    LB: Builder<L> + Clone,
    H: Hasher + Clone,
    RB: Builder<R>,
{
}

impl<C, H, B> Builder<Associative<C, MultisetHasher<H>>>
    for AssociativeBuilder<C, H, B>
where
    B: Builder<C> + Clone,
    H: Hasher + Clone,
{
    fn build(self) -> Associative<C, MultisetHasher<H>> {
        Associative::new(
            vec![self.builder.clone().build()],
            self.set_hasher,
        )
    }
}

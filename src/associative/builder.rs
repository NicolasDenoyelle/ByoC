use crate::associative::ExclusiveHasher;
use crate::builder::Build;
use crate::Associative;
use std::hash::Hasher;
use std::marker::PhantomData;

/// `Associative` container builder.
///
/// This builder can be consumed later to wrap some containers into an
/// [`Associative`](../../struct.Associative.html) container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Associative};
/// use byoc::builder::{AssociativeBuild, Build};
/// use byoc::builder::builders::{ArrayBuilder, AssociativeBuilder};
/// use std::collections::hash_map::DefaultHasher;
///
/// let array_builder = ArrayBuilder::new(2);
/// let mut container: Associative::<Array<(u64,u64)>,_> =
///     AssociativeBuilder::<_,_,_>::new(array_builder,
///                                      DefaultHasher::new(), 2).build();
/// assert_eq!(container.capacity(), 4);
/// let associative: Associative::<Array<(u64,u64)>,_> =
///                  Associative::new(vec![Array::new(2), Array::new(2)],
///                                   DefaultHasher::new());
/// assert_eq!(associative.capacity(), 4);
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
    B: Build<C> + Clone,
{
    builder: B,
    set_hasher: ExclusiveHasher<H>,
    unused: PhantomData<C>,
}

impl<C, H, B> Clone for AssociativeBuilder<C, H, B>
where
    H: Hasher + Clone,
    B: Build<C> + Clone,
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
    B: Build<C> + Clone,
{
    pub fn new(builder: B, key_hasher: H, num_sets: usize) -> Self {
        AssociativeBuilder {
            builder,
            set_hasher: ExclusiveHasher::new(key_hasher, num_sets),
            unused: PhantomData,
        }
    }

    pub fn add_layer(
        self,
        num_keys: usize,
    ) -> AssociativeBuilder<Associative<C, ExclusiveHasher<H>>, H, Self>
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

impl<C, H, B> Build<Associative<C, ExclusiveHasher<H>>>
    for AssociativeBuilder<C, H, B>
where
    B: Build<C> + Clone,
    H: Hasher + Clone,
{
    fn build(self) -> Associative<C, ExclusiveHasher<H>> {
        Associative::new(
            vec![self.builder.clone().build()],
            self.set_hasher,
        )
    }
}
